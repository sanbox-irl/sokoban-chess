#[allow(dead_code)]
pub fn wrap_f32(basic: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    basic - f32::floor(basic / range) * range + min
}

#[allow(dead_code)]
pub fn wrap_usize(basic: i32, min: i32, max: i32) -> i32 {
    let range = max - min;
    basic - (f32::floor(basic as f32 / range as f32) as i32) * range + min
}
