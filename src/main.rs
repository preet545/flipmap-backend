use futures::StreamExt;
use serde::Serialize;
use std::{env, vec};
use std::io::{self, BufRead};
use std::time::Duration;
use futures::stream::FuturesUnordered; // Feels like I should be able to use tokio to do this??

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

// An interactive proof-of-concept for making external API calls
async fn poc_menu() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdin = io::stdin();
    let mut buffer = String::with_capacity(2048); // Maybe a lot for a line?

    loop {
        println!("Choose an option: 1) Route query 2) Geocode query (Ctrl+D to exit)");
        stdin.lock().read_line(&mut buffer)?; //TODO: do I need to care about unlocking?

        match buffer.trim() {
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
            "2" => {
                // TODO: This!
                println!("Enter a single coordinate as (lon,lat):");
                let mut coord_input = String::new();
                stdin.read_line(&mut coord_input)?;
                // Here you would parse the input and call geocode_request
                // For now, just print the input
                println!("Geocode query with coordinate: {}", coord_input);
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
    //TODO: Coordinates can (should?) be waypoint vecs, not just start-end
    coordinates: [geo_json::Coordinate; 2],
    instructions: bool,
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
}
