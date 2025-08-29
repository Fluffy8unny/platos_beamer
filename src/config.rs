use crate::types::SubtractorType;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use toml;

#[derive(Deserialize)]
pub struct CameraConfig {
    pub device_index: i32,
}

#[derive(Deserialize)]
pub struct BgSubConfig {
    pub subtractor_type: SubtractorType,
}

#[derive(Deserialize)]
pub struct PlatoConfig {
    pub camera_config: CameraConfig,
    pub background_subtractor_config: BgSubConfig,
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
