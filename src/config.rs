use crate::types::SubtractorType;
use serde::Deserialize;
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

pub fn load_config(path: &str) -> Result<PlatoConfig, Box<dyn std::error::Error>> {
    let contents = match std::fs::read_to_string(path) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Could not load config at {}. Terminating.", path);
            return Err(Box::new(e));
        }
    };
    let config: PlatoConfig = toml::from_str(&contents)?;
    Ok(config)
}
