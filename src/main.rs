use geo_json::Coordinate;
use serde::Serialize;
use std::{env, vec};
use std::io::{self, BufRead};
use std::time::Duration;

mod geo_json;
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),);
// I think the struct is thread safe (reqwest client has an arc internally) but the compiler 
// is really mad if I don't use sync stuff to make this static
// TODO: Initialize this in a less dirty/global way later on
use std::sync::LazyLock;
static CLIENT: LazyLock<ExternalRequester> = LazyLock::new(|| {ExternalRequester::new()});


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dbg!(poc_menu().await?);
    Ok(())
}

// Unlikely to use these outside demo
use futures::StreamExt;
use futures::stream::FuturesUnordered; // Feels like I should be able to use tokio to do this??
use regex::Regex;
// An interactive proof-of-concept for making external API calls
async fn poc_menu() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdin = io::stdin();
    let mut buffer = String::with_capacity(2048);

    loop {
        println!("Choose an option:\n  1) Route query\n  2) Reverse Geocode query\n  3) Geocode query\n  (Ctrl+D to exit)");
        stdin.lock().read_line(&mut buffer)?;

        match buffer.trim() {
            // Route Query
            "1" => {
                // Demonstrate running a few requests at once and waiting async for the joins
                // OSU to PDX
                let req1 = OpenRouteRequest{
                    coordinates: [
                        geo_json::Coordinate{lat: 45.528104715146554, lon: -122.67683019518431},
                        geo_json::Coordinate{lat: 44.56580672743879, lon: -123.28215624028414}],
                    instructions: false
                };
                // Atlanta to Savannah. Do it again?
                let req2 = OpenRouteRequest{
                    coordinates: [
                        geo_json::Coordinate{lat: 33.756944444444, lon: -84.390277777778},
                        geo_json::Coordinate{lat: 32.050833333333, lon: -81.103888888889}],
                    instructions: false
                };
                // API Doc Example
                let req3 = OpenRouteRequest{
                    coordinates: [
                        geo_json::Coordinate{lat: 49.41461, lon: 8.681495},
                        geo_json::Coordinate{lat: 49.420318, lon: 8.687872}],
                    instructions: false
                };
                let reqs = vec![req1,req2,req3];

                /* Body problems test
                let req = CLIENT.ors(&reqs[0]).build()?;
                let body = String::from_utf8_lossy(req.body().unwrap().as_bytes().unwrap());
                dbg!(body);
                */

                // Why's this 2 parts?: 
                //  .send() shoots off an async web request. Reqwest handles the pool
                //
                //  resps.next().await polls for responses and grabs whatever finishes first
                //  (instead of blocking until a given req is ready)
                let mut resps: FuturesUnordered<_> = reqs.into_iter().map(|req| {
                    let client = &CLIENT;
                    client.ors(&req).send()
                }).collect();

                while let Some(resp) = resps.next().await {
                    // I'm kind of confused about how the Option is wrapping a Result here
                    let extremely_resp = resp.unwrap(); // idk rn tbh
                    dbg!(&extremely_resp);
                    /* I'll spare you the heap of coordinates
                    dbg!(&extremely_resp.text().await?); 
                    */
                }
            }
            // Reverse Geocode
            "2" => {
                println!("Enter a single coordinate as (lon,lat) or nothing for prefilled test:");
                buffer.clear();
                stdin.read_line(&mut buffer)?;
                // Try to regex out some coordinates or just use the computing center
                const MILNE: geo_json::Coordinate = geo_json::Coordinate{
                    lat: 44.566388,
                    lon: -123.275304,}; 
                let re = Regex::new(r"\((-?\d{1,3}\.\d*),(-?\d{1,3}\.\d*)\)")?;
                let coord: geo_json::Coordinate = re.captures(&buffer).and_then(|coords| {
                    let lon = coords.get(1)?.as_str().parse::<f64>().ok()?;
                    let lat = coords.get(2)?.as_str().parse::<f64>().ok()?;
                    Some(Coordinate{lat,lon})}).unwrap_or(MILNE);
                let res = CLIENT.photon_reverse(coord).send().await?;
                dbg!(&res);
                dbg!(&res.text().await?);
            }
            // Geocode
            "3" => {
                const ANCHOR_LAT: f64 = 44.56580672743879;
                const ANCHOR_LON: f64 = -123.28215624028414;

                println!("Enter a place name or nothing for prefilled test:\n    (search anchored @ OSU)");
                buffer.clear();
                stdin.read_line(&mut buffer)?;
                let q = Some(buffer.clone().trim()).filter(|s| !s.is_empty()).unwrap_or("Downward Dog").to_string();
                let req = PhotonGeocodeRequest{
                    limit: 5,
                    query: q,
                    lat: Some(ANCHOR_LAT),
                    lon: Some(ANCHOR_LON),
                };
                let res = CLIENT.photon(&req).send().await?;
                dbg!(&res);
                dbg!(&res.text().await?);
            }
            "" => {
                // I think only EOF will make this happen
                println!("Goodbye!");
                return Ok(());
            }
            _ => println!("Invalid option, please try again."),
        }
        buffer.clear();
    }
}

// TODO: Constructor maybe
#[derive(Serialize)]
struct OpenRouteRequest {
    //TODO: Use geojson types to more closely follow 'Position' type
    coordinates: [geo_json::Coordinate; 2],
    instructions: bool,
}

#[derive(Serialize)]
struct PhotonGeocodeRequest { 
    limit: u8, // Probably just 1 for "where am I" and ~10 for a search
    #[serde(rename(serialize="q"))]
    query: String, // Might be possible to use str here
    // TODO: Quick and dirty optional 'anchor' here 
    // in the future we'll use a geojson type with proper deserialization
    lat: Option<f64>,
    lon: Option<f64>,
}

pub struct ExternalRequester {
    client: reqwest::Client,
    open_route_service_key: String,
    // We also use Photon (via Komoot) but it has an unauthenticated API
}

// Wrapper around Reqwest that tries to ape the same client/req-builder API
// For use in making Open Route Service (routing only) or Photon (via Komoot) calls
impl ExternalRequester {
    fn new() -> Self {
    ExternalRequester {
            client: 
                reqwest::Client::builder()
                    .user_agent(USER_AGENT)
                    .timeout(Duration::from_secs(10))
                    .https_only(true)
                    .build()
                    .expect("req client construction failed"),
            open_route_service_key:
                // TODO: Allow reading from a file too and other such logic
                env::var("ORS_API_KEY")
                    .expect("Place an Open Route Service API key in ORS_API_KEY env variable!")
                    .to_string(),
    }
}
    // TODO: Re-evaluate if this is useful here. This just makes futures and doesn't execute
    fn ors(&self, req: &OpenRouteRequest) -> reqwest::RequestBuilder {
        self.client.post("https://api.openrouteservice.org/v2/directions/driving-car/geojson")
            .header("Content-Type", "application/json")
            .header("Authorization", &self.open_route_service_key)
            .json(req)
    }

    fn photon_reverse(&self, coord: geo_json::Coordinate) -> reqwest::RequestBuilder {
        // TODO: This sucks but we've got a different serializer for ORS already. We'll use
        // geojson types in the future anyway
        let q = [("lon",coord.lon),("lat",coord.lat)];
        self.client.get("https://photon.komoot.io/reverse").query(&q)
    }

    fn photon(&self, req: &PhotonGeocodeRequest) -> reqwest::RequestBuilder {
        self.client.get("https://photon.komoot.io/api/").query(req)
    }
}
