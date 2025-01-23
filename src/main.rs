// mod geo_json;
use std::env;
use std::io::{Read, Write};
use tokio::net::TcpStream;

// use geo::LineString;
use hello_osm::map_range;

// TODO phase this out and receive coordinates
const USER_LAT: &str = "44.560975";
const USER_LON: &str = "-123.275227";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}
