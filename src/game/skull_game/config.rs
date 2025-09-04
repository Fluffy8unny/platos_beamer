use crate::config::open_file;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use toml;

#[derive(Deserialize, Clone)]
pub struct SkullSettings {
    pub spawn_rate: f32,
}
