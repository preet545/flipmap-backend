use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use std::env;
use geojson::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
mod consts;
mod requester;

use crate::requester::{ExternalRequester, OpenRouteRequest, PhotonGeocodeRequest};
// I think the struct is thread safe (reqwest client has an arc internally) but the compiler
// is really mad if I don't use sync stuff to make this static
// TODO: Initialize this in a less dirty/global way later on
use axum::extract::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Arc::new(ExternalRequester::new());
    //const CLIENT: LazyLock<ExternalRequester> = LazyLock::new(|| ExternalRequester::new());
    //let CLIENT: ExternalRequester = ExternalRequester::new();
    let app: Router = Router::new()
        .route("/route", post(route))
        .with_state(client);

    // Take an IP address and a port number from the command line instead of this static str. Then,
    // print it before starting the listener AI!
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <IP> <PORT>", args[0]);
        std::process::exit(1);
    }
    let ip = &args[1];
    let port = &args[2];
    println!("Starting server on {}:{}", ip, port);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ip, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
    // AI: Removed AI comment
}

#[derive(Deserialize)]
pub struct RouteRequest {
    pub lat: f64,
    pub lon: f64,
    pub query: String,
}

#[derive(Serialize)]
pub struct RouteResponse {
    // A position vec, but concatenated so it's just vec<num...> instead of vec<vec<num..>..>
    pub route: Vec<f64>,
}

#[axum::debug_handler]
async fn route(
    State(client): State<Arc<ExternalRequester>>,
    Json(params): Json<RouteRequest>,
) -> impl IntoResponse {
    /*
    // Photon will also do this (and identify the wrong param) but let's fail fast
    // TODO: May or may not be preferable to do this during deserialization??
    if (params.lat < -90.0 || params.lat > 90.0) || (params.lon < -180.0 && params.lon > 180.0) {
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "One or both parameters out of range: lat:{}, lon:{}",
                params.lat, params.lon
            ),
        );
    }
    */

    // First request to know where to ask for the route's end waypoint
    // TODO: Handle the MANY possible errors w.r.t req and parsing
    let req = PhotonGeocodeRequest {
        lat: Some(params.lat),
        lon: Some(params.lon),
        limit: 1,
        query: params.query,
    };
    let res: reqwest::Response = client.photon(&req).send().await.unwrap();
    let res_features = res.json::<geojson::FeatureCollection>().await.unwrap();
    // All we want is the coordinates of the point. FeatureCollection -> Feature -> Point
    let end_coord: Position = match &res_features.features[0].geometry.as_ref().unwrap().value {
        geojson::Value::Point(x) => x.clone(),
        _ => panic!("Got non-position geometry value"),
    };

    // Second request to actually get the route
    let start_coord: Position = vec![params.lon, params.lat];
    let req = OpenRouteRequest {
        instructions: false,
        coordinates: vec![start_coord, end_coord],
    };
    let res: reqwest::Response = client.ors(&req).send().await.unwrap();
    let res_features = res.json::<geojson::FeatureCollection>().await.unwrap();

    // I call this the 'sausage factory'
    let resp = RouteResponse {
        route: res_features
            .into_iter()
            .filter_map(|feat| feat.geometry)
            .filter_map(|geo| match geo.value {
                geojson::Value::Point(pt) => Some(pt),
                _ => None,
            })
            .collect::<Vec<Vec<f64>>>()
            .into_iter()
            .flatten()
            .collect(),
    };
    (StatusCode::OK, Json(resp))
}
