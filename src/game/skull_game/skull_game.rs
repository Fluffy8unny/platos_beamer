use crate::PlatoConfig;
use crate::display::{display_window::DisplayType, timestep::TimeStep};

use glium::winit::keyboard::Key;
use opencv::prelude::*;

pub struct SkullGame {}

impl SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        _config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        _timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
