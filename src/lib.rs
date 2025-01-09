use std::ops::{Div, Mul, Add, Sub};

// map value from a range of [old_min, old_max] to [new_min, new_max]
pub fn map_range<T>(value: T, old_min: T, old_max: T, new_min: T, new_max: T) -> T 
    where  T: Add<Output = T> + Sub<Output = T> + Mul<Output  = T> + Div<Output = T> + Copy
{
    new_min + (value - old_min) * (new_max - new_min) / (old_max - old_min)
}

pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}
