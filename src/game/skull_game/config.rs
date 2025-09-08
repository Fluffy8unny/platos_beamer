use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct SkullSettings {
    pub crystal_position: (f32, f32),
    pub spawn_rate: f32,
    pub max_number: usize,
    pub scale_speed: f32,
    pub move_speed: f32,
    pub threshold: f32,
    pub hitable_from: f32,
    pub start_scale: f32,
    pub max_scale: f32,
    pub x_start: (f32, f32),
    pub y_start: (f32, f32),
    pub rot: (f32, f32),
}
