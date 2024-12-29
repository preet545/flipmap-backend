use std::ops::{Div, Mul, Add, Sub};

mod geoJson;
mod graphics;

fn map_range<T>(value: T, from_min: T, from_max: T, to_min: T, to_max: T) -> T 
    where  T: Add<Output = T> + Sub<Output = T> + Mul<Output  = T> + Div<Output = T> + Copy
{
    to_min + (value - from_min) * (to_max - to_min) / (from_max - from_min)
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn main() {
    let route_str = std::fs::read_to_string("data/route2.json").unwrap();
    let route_json : geoJson::GeoJson = serde_json::from_str(&route_str).unwrap();

    /*
     Some(
       Geometry {
         coordinates: [
           [ -122.654622, 45.523775, ],
           [ -122.654623, 45.523622, ],
                         ...
    */
    // let points : &Option<geoJson::Geometry> = &route_json.features.unwrap()[0].geometry;
    let features = &route_json.features.unwrap();
    let geometry = features[0].geometry.as_ref().unwrap();
    let points = &geometry.coordinates;

    // println!("{:?}", points);
    let bbox = features[0].bbox;

    let scaled_points : Vec<(f64, f64)> = points.iter().map(|(x, y)|
        (map_range(*x, bbox.0, bbox.2, -1.0, 1.0), map_range(*y, bbox.1, bbox.3, -1.0, 1.0))
    ).collect();
    // println!("{:?}", scaled_points);


    // convert lon,lat to x,y (how? haversine based on bounding?)

    pollster::block_on(graphics::run(&scaled_points));



    // render all points
    // render all lines
    // pan around on canvas
}
