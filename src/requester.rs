use std::env;
use std::time::Duration;
use serde::Serialize;

use crate::consts;

// TODO: Constructor for both these maybe
#[derive(Serialize)]
pub struct OpenRouteRequest {
    pub coordinates: Vec<geojson::Position>,
    pub instructions: bool,
}

#[derive(Serialize)]
pub struct PhotonGeocodeRequest { 
    pub limit: u8, // Probably just 1 for "where am I" and ~10 for a search
    #[serde(rename(serialize="q"))]
    pub query: String, // Might be possible to use str here
    // TODO: Quick and dirty optional 'anchor' here 
    // in the future we'll use a geojson type with proper deserialization
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Serialize)]
pub struct PhotonRevGeocodeRequest {
    pub lat: f64,
    pub lon: f64,
}

pub struct ExternalRequester {
    client: reqwest::Client,
    open_route_service_key: String,
    // We also use Photon (via Komoot) but it has an unauthenticated API
}

// Wrapper around Reqwest that tries to ape the same client/req-builder API
// For use in making Open Route Service (routing only) or Photon (via Komoot) calls
impl ExternalRequester {
    pub fn new() -> Self {
    ExternalRequester {
            client: 
                reqwest::Client::builder()
                    .user_agent(consts::USER_AGENT)
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
    pub fn ors(&self, req: &OpenRouteRequest) -> reqwest::RequestBuilder {
        self.client.post("https://api.openrouteservice.org/v2/directions/driving-car/geojson")
            .header("Content-Type", "application/json")
            .header("Authorization", &self.open_route_service_key)
            .json(req)
    }

    pub fn photon_reverse(&self, coord: PhotonRevGeocodeRequest) -> reqwest::RequestBuilder {
        let q = [("lon",coord.lon),("lat",coord.lat)];
        self.client.get("https://photon.komoot.io/reverse").query(&q)
    }

    pub fn photon(&self, req: &PhotonGeocodeRequest) -> reqwest::RequestBuilder {
        self.client.get("https://photon.komoot.io/api/").query(req)
    }
}
