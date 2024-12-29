mod geoJson;
mod graphics;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn main() {
    let route_str = std::fs::read_to_string("data/route.json").unwrap();
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
    let points : &geoJson::Geometry = features[0].geometry.as_ref().unwrap();

    println!("{:?}", points);

    pollster::block_on(graphics::run());


    // convert lon,lat to x,y (how? haversine based on centroid?)

    // render all points
    // render all lines
    // pan around on canvas
}
