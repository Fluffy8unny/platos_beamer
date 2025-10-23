use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct SkullSettings {
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
    pub mask_color: (u8, u8, u8),
    pub skull_shader: String,
    pub particle_shader: String,
    pub moon_shader: String,
    pub victory_shader: String,
    pub moon_textures: Vec<String>,
    pub skull_alive_textures: Vec<String>,
    pub skull_killed_textures: Vec<String>,
    pub victory_texture: String,
    pub skull_killed_sound: String,
    pub skull_escaped_sound: String,
    pub start_key: String,
}
