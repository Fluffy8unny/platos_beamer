use crate::{
    config::PlatoConfig, display::display_window::DisplayType, display::timestep::TimeStep,
};
use glium::winit::keyboard::Key;
use opencv::prelude::*;

pub trait GameTrait {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn update(
        &mut self,
        mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn key_event(&mut self, event: &Key);
    fn reset(&mut self);
}
