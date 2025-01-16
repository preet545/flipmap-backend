// mod geo_json;
mod graphics;

use std::io::{Read, Write};
use reqwest;
use std::env;

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

    // // convert lon,lat to x,y (TODO update to Haversine distance)
    // What happens if we cross the meridian/antimeridian? What about the poles (probably no routes
    // thru those tho)
    // ----*
    // |   |
    // |   |
    // *----
    // GeoJSON bbox is (sw, ne) btw
    let scaled_route : Vec<(f64, f64)> = points.iter().map(|coord|
        (map_range(coord[0], bbox[0], bbox[2], -1.0, 1.0),
         map_range(coord[1], bbox[1], bbox[3], -1.0, 1.0))
    ).collect();


    pollster::block_on(graphics::run(&scaled_route));
}
