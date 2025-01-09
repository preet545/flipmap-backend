mod geo_json;
mod graphics;

use hello_osm::map_range;
use std::{error::Error, io::{self, stdout, Read, Write}};

fn main() {
    let current_lat = "44.55955383903737";
    let current_lon = "-123.27702371574132";
    let mut destination = String::new();

    // print!("desired destination: ");
    // let _ = io::stdout().flush();
    // io::stdin().read_line(&mut destination).expect("couldn't save that string");

    // let mut url = reqwest::Url::parse("https://photon.komoot.io/api/").expect("broken");
    // let params = [("q", destination.to_string()), ("lat", current_lat.to_string()), ("lon", current_lon.to_string())];
    // for (key, val) in params { url.query_pairs_mut().append_pair(&key, &val); }

    // let mut body = String::new();
    // let _ = reqwest::blocking::get(url).unwrap().read_to_string(&mut body);

    // println!("body = {body:?}");

    // PLACEHOLDER
    let route_str = std::fs::read_to_string("data/route2.json").unwrap();
    let search_results_str = std::fs::read_to_string("data/photon.json").unwrap();

    let route_json : geo_json::GeoJson = serde_json::from_str(&route_str).unwrap();
    let search_json : geo_json::GeoJson = serde_json::from_str(&route_str).unwrap();

    let features = &route_json.features.unwrap();
    let geometry = features[0].geometry.as_ref().unwrap();
    let points = &geometry.coordinates;
    let bbox = &features[0].bbox;

    // convert lon,lat to x,y (TODO update to Haversine distance)
    let scaled_points = points.iter().map(|coord|
        (map_range(coord.lat, bbox.0, bbox.2, -1.0, 1.0),
         map_range(coord.lon, bbox.1, bbox.3, -1.0, 1.0))
    ).collect();

    pollster::block_on(graphics::run(&scaled_points));
}
