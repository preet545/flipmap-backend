//      lat,               lon
// PDX: 45.528104715146554, -122.67683019518431
// OSU: 44.56580672743879,  -123.28215624028414

use serde::{ser::SerializeTuple, Deserialize, Serialize};

// TODO make coordinate struct
#[derive(Deserialize, Debug)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

// We want typed fields to not mix things up, but Open Route Service wants an array with 2
// entries per pair. TODO: Does this change outside the /v2/directions endpoint?
impl Serialize for Coordinate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // openrouteservice API flips it to lon, lat :(
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element(&self.lon)?;
        seq.serialize_element(&self.lat)?;
        seq.end()
    }
}

// TODO figure out how to parse this
// (called `Result::unwrap()` on an `Err` value: Error("invalid type: floating point `-123.279945`, expected struct Coordinate", line: 1, column: 47))
#[derive(Deserialize, Debug)]
pub struct Bbox {
    pub nw: Coordinate,
    pub se: Coordinate,
}

#[derive(Deserialize, Debug)]
pub enum Type {
    Geometry,
    Feature,
    FeatureCollection,
    LineString,
    Point,
}

#[derive(Deserialize, Debug)]
pub enum GeometryCoordinates {
    LineString(Vec<Coordinate>),
    Point(Coordinate),
}

#[derive(Debug, Deserialize)]
pub struct GeoJson {
    #[serde(rename = "type")]
    pub obj_type: Type,
    pub bbox: (f64, f64, f64, f64),
    pub features: Option<Vec<GeoJson>>,
    pub properties: Option<Properties>,
    pub geometry: Option<Geometry>,
    // ignoring for now
    // pub metadata:
}

// TODO investigate what's going on with bbox parsing
#[derive(Debug, Deserialize)]
pub struct Feature {
    pub properties: String,
    pub bbox: (Coordinate, Coordinate),
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    pub segments: Vec<Segment>,
    pub way_points: (u32, u32),
    // summary: Summary,
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    pub coordinates: GeometryCoordinates,
}

#[derive(Debug, Deserialize)]
pub struct Segment {
    distance: f32,
    duration: f32,
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    pub distance: f32,
    pub duration: f32,
    #[serde(rename = "type")]
    pub instruction_type: u32,
    pub instruction: String,
    pub name: String,
    pub way_points: (u32, u32),
}
