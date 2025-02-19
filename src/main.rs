// mod geo_json;
mod graphics;

use std::io::{Read, Write};
use reqwest;
use std::env;

//use std::io::{Read, Write};
use std::collections::HashMap;
use warp::Filter;
use serde::{Deserialize, Serialize};
// use geo::LineString;
use hello_osm::map_range;

// TODO phase this out and receive coordinates
const USER_LAT : &str = "44.560975";
const USER_LON : &str = "-123.275227";

fn main() {
    let mut destination = String::new();
    
    // GET DESTINATION
    print!("desired destination: ");
    let _ = std::io::stdout().flush();
    std::io::stdin().read_line(&mut destination).expect("couldn't save that string");

    let mut url = reqwest::Url::parse("https://photon.komoot.io/api/").expect("broken");
    let params = [("q", destination.to_string()), ("lat", USER_LAT.to_string()), ("lon", USER_LON.to_string())];
    for (key, val) in params { url.query_pairs_mut().append_pair(&key, &val); }

    let mut search_results_str = String::new();
    let _ = reqwest::blocking::get(url).unwrap().read_to_string(&mut search_results_str);

    // SEARCH (DESTINATION) PARSING
    let search_json : geojson::FeatureCollection = serde_json::from_str(&search_results_str).unwrap();

    // let results = &search_json.features[0].properties;
    let result_coords = match &search_json.features[0].geometry.as_ref().unwrap().value {
        geojson::Value::Point(x) => x,
        _ => panic!("Destination shold be a Point"),
    };
    println!("Found at {}, {}", result_coords[1], result_coords[0]);

    let openroute_key = env::var("OPENROUTE_API_KEY").unwrap();
    let mut url = reqwest::Url::parse("https://api.openrouteservice.org/v2/directions/driving-car").expect("broken");
    let start_str = USER_LON.to_owned() + "," + USER_LAT;
    let end_str = result_coords[0].to_string() + "," + &result_coords[1].to_string();
    let params = [("api_key", openroute_key.to_string()), ("start", start_str.to_string()), ("end", end_str.to_string())];
    for (key, val) in params { url.query_pairs_mut().append_pair(&key, &val); }

    // ROUTING
    let mut route_str = String::new();
    let _ = reqwest::blocking::get(url).unwrap().read_to_string(&mut route_str);

    let route_json : geojson::FeatureCollection = serde_json::from_str(&route_str).unwrap();

    let features = &route_json.features;
    let geometry = features[0].geometry.as_ref().unwrap();
    let points = match &geometry.value {
        geojson::Value::LineString(coords) => coords,
        _ => panic!("route should be a LineString")
    };
    let bbox = features[0].bbox.as_ref().expect("how did u lose the bbox");

    // // convert lon,lat to x,y 
    // What happens if we cross the meridian/antimeridian? What about the poles (probably no routes
    // thru those tho)
    // GeoJSON bbox is (sw, ne) btw
    let scaled_route : Vec<(f64, f64)> = points.iter().map(|coord|
        (map_range(coord[0], bbox[0], bbox[2], -1.0, 1.0),
         map_range(coord[1], bbox[1], bbox[3], -1.0, 1.0))
    ).collect();


    pollster::block_on(graphics::run(&scaled_route));
}


/// Preet Patel 
mod graphics;

// Struct to store multiple search results
#[derive(Serialize, Deserialize)]
struct Location {
    name: String,    // Place name
    lat: f64,        // Latitude
    lon: f64,        // Longitude
    address: Option<String>, // Address 

// Function to fetch multiple places from Photon API
fn fetch_places(query: &str) -> Vec<Location> {
    let mut url = reqwest::Url::parse("https://photon.komoot.io/api/").expect("Invalid URL");

    let params = [("q", query.to_string()), ("lat", USER_LAT.to_string()), ("lon", USER_LON.to_string())];
    for (key, val) in params { 
        url.query_pairs_mut().append_pair(&key, &val);
    }

    let mut search_results_str = String::new();
    reqwest::blocking::get(url)
        .expect("API request failed")
        .read_to_string(&mut search_results_str)
        .expect("Failed to read response");

    let search_json: geojson::FeatureCollection = serde_json::from_str(&search_results_str)
        .expect("Failed to parse JSON");

    let mut locations = Vec::new(); // Store multiple locations

    // Extract multiple destinations
    for feature in search_json.features {
        if let Some(geojson::Value::Point(coords)) = feature.geometry.map(|g| g.value) {
            let props = feature.properties.unwrap_or_else(|| HashMap::new());
            let name = props.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let address = props.get("address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            locations.push(Location {
                name,
                lat: coords[1], // Latitude
                lon: coords[0], // Longitude
                address,
            });
        }
    }

    locations
}

// Function to find the route using OpenRouteService
fn fetch_route(start_lat: &str, start_lon: &str, dest_lat: &str, dest_lon: &str) -> geojson::FeatureCollection {
    let openroute_key = env::var("OPENROUTE_API_KEY").expect("Missing API Key");
    let mut url = reqwest::Url::parse("https://api.openrouteservice.org/v2/directions/driving-car").expect("Broken URL");

    let params = [
        ("api_key", openroute_key.to_string()), 
        ("start", format!("{},{}", start_lon, start_lat)), 
        ("end", format!("{},{}", dest_lon, dest_lat))
    ];
    for (key, val) in params { 
        url.query_pairs_mut().append_pair(&key, &val);
    }

    let mut route_str = String::new();
    reqwest::blocking::get(url).unwrap().read_to_string(&mut route_str).expect("Failed to fetch route");

    serde_json::from_str(&route_str).expect("Failed to parse route JSON")
}

// Search API handler
async fn search_handler(query: String) -> warp::reply::Json {
    let results = fetch_places(&query);
    warp::reply::json(&results)
}

#[tokio::main]
async fn main() {
    // Ask user for destination
    let mut destination = String::new();
    print!("Desired destination: ");
    let _ = std::io::stdout().flush();
    std::io::stdin().read_line(&mut destination).expect("Failed to read input");

    let places = fetch_places(destination.trim());

    if places.is_empty() {
        println!("No locations found for '{}'", destination.trim());
        return;
    }

    // Display multiple results
    println!("Found locations:");
    for (i, place) in places.iter().enumerate() {
        println!("{}: {} at ({}, {})", i + 1, place.name, place.lat, place.lon);
    }

    // Select the first location for routing
    let first_location = &places[0];

    // Fetch route from USER location to first found place
    let route_json = fetch_route(USER_LAT, USER_LON, &first_location.lat.to_string(), &first_location.lon.to_string());

    let features = &route_json.features;
    let geometry = features[0].geometry.as_ref().unwrap();
    let points = match &geometry.value {
        geojson::Value::LineString(coords) => coords,
        _ => panic!("Route should be a LineString"),
    };
    let bbox = features[0].bbox.as_ref().expect("Bounding box missing");

    // Scale route coordinates for visualization
    let scaled_route: Vec<(f64, f64)> = points.iter().map(|coord|
        (map_range(coord[0], bbox[0], bbox[2], -1.0, 1.0),
         map_range(coord[1], bbox[1], bbox[3], -1.0, 1.0))
    ).collect();

    pollster::block_on(graphics::run(&scaled_route));

    // Start HTTP API server
    let search_route = warp::path!("search" / String)
        .and(warp::get())
        .and_then(|query: String| async move { 
            Ok::<_, warp::Rejection>(search_handler(query).await) 
        });

    println!("Server running on http://127.0.0.1:3030/");

    warp::serve(search_route).run(([127, 0, 0, 1], 3030)).await;
}
