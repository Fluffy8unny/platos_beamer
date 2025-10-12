use crate::{
    config::PlatoConfig, display::display_window::DisplayType, display::timestep::TimeStep,
};

use glium::winit::keyboard::Key;
use opencv::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub enum GameType {
    IdentityGame,
    SkullGame,
    CalibrationGame,
}

pub trait GameTrait {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn update(
        &mut self,
        _image: &Mat,
        _mask: &Mat,
        _display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        display: &DisplayType,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn key_event(&mut self, _event: &Key) {}

    fn reset(&mut self) {}
}
