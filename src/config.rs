use crate::types::SubtractorType;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use toml;

#[derive(Deserialize, Clone)]
pub struct CameraConfig {
    pub device_index: i32,
}

#[derive(Deserialize, Clone)]
pub struct BgSubConfig {
    pub subtractor_type: SubtractorType,
}

#[derive(Deserialize, Clone)]
pub struct MinimapConfig {
    pub show: bool,
    pub position: (f32, f32),
    pub dims: (f32, f32),
}

#[derive(Deserialize, Clone)]
pub struct KeyConfig {
    pub quit_key: String,
    pub reset_key: String,
    pub toggle_minimap_key: String,
}

#[derive(Deserialize, Clone)]
pub struct PlatoConfig {
    pub camera_config: CameraConfig,
    pub background_subtractor_config: BgSubConfig,
    pub minimap_config: MinimapConfig,
    pub key_config: KeyConfig,
}

#[derive(Deserialize)]
pub struct ShaderConfig {
    pub name: String,
    pub vertex: String,
    pub fragment: String,
}

fn open_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    match std::fs::read_to_string(path) {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Could not load config at {}. Terminating.", path);
            return Err(Box::new(e));
        }
    }
}

pub fn load_config<T: DeserializeOwned>(path: &str) -> Result<T, Box<dyn std::error::Error>> {
    let contents = open_file(path)?;
    Ok(toml::from_str(&contents)?)
}
