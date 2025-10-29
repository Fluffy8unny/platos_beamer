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
    pub scale: (f32, f32),
    pub color_overlay: Vec<[f32; 3]>,
    pub corona_color: Vec<[f32; 3]>,
    pub life_interpolator: f32,
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
    pub erratic_movement: f32,
}

#[derive(Deserialize, Clone)]
pub struct KeySettings {
    pub start_key: String,
    pub easy_mode_key: String,
    pub normal_mode_key: String,
}

#[derive(Deserialize, Clone, Copy)]
pub struct DifficultySelector {
    pub player_damage: u32,
    pub escape_penalty: u32,
}

#[derive(Deserialize, Clone)]
pub struct DifficultySettings {
    pub easy: DifficultySelector,
    pub normal: DifficultySelector,
}

#[derive(Deserialize, Clone)]
pub struct GameSettings {
    pub skull_settings: SkullSettings,
    pub moon_settings: MoonSettings,
    pub particle_settings: ParticleSettings,
    pub shader_settings: ListToLoad,
    pub sound_settings: ListToLoad,
    pub key_settings: KeySettings,
    pub difficultiy_settings: DifficultySettings,
    pub texture_settings: TextureSettings,
    pub number_of_rounds: u32,
    pub number_of_kill_sounds: u32,
    pub number_of_escape_sounds: u32,
}

pub fn valdiate_config(settings: &GameSettings) -> Result<(), Box<dyn std::error::Error>> {
    let number_of_rounds = (settings.number_of_rounds + 1) as usize;
    let number_of_overlays = settings.moon_settings.color_overlay.len();
    let number_of_coorona_colors = settings.moon_settings.corona_color.len();

    if number_of_rounds != number_of_overlays {
        return Err(format!(
            "number of color overalys{} != number of rounds {}",
            number_of_overlays, number_of_rounds
        )
        .into());
    }

    if number_of_rounds != number_of_coorona_colors {
        return Err(format!(
            "number of corona colors{} != number of rounds {}",
            number_of_coorona_colors, number_of_rounds
        )
        .into());
    }

    //todo check if all sounds are there
    Ok(())
}
