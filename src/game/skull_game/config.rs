use serde::Deserialize;

type ListToLoad = Vec<(String, String)>;

#[derive(Deserialize, Clone)]
pub struct ParticleSetting {
    pub scale: f32,
    pub color: (f32, f32, f32),
    pub opacity: f32,
    pub initial_velocity: f32,
    pub number: usize,
}
#[derive(Deserialize, Clone)]
pub struct ParticleSettings {
    pub escaped: ParticleSetting,
    pub killed: ParticleSetting,
    pub visualization: ParticleSetting,
}

#[derive(Deserialize, Clone)]
pub struct MoonSettings {
    pub starting_life: u32,
    pub position: (f32, f32),
    pub max_position: (f32, f32),
    pub scale: f32,
}

#[derive(Deserialize, Clone)]
pub struct TextureSettings {
    pub mask_color: (u8, u8, u8),
    pub texture_arrays: Vec<(String, Vec<String>)>,
    pub textures: ListToLoad,
}

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
}

#[derive(Deserialize, Clone)]
pub struct KeySettings {
    pub start_key: String,
    pub easy_mode_key: String,
    pub normal_mode_key: String,
}

#[derive(Deserialize, Clone)]
pub struct GameSettings {
    pub skull_settings: SkullSettings,
    pub moon_settings: MoonSettings,
    pub particle_settings: ParticleSettings,
    pub shader_settings: ListToLoad,
    pub sound_settings: ListToLoad,
    pub key_settings: KeySettings,
    pub texture_settings: TextureSettings,
}
