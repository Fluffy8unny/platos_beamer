use crate::PlatoConfig;
use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::skull::Skull;
use crate::types::game_types::GameTrait;

use ::glium::{IndexBuffer, VertexBuffer};
use glium::implement_vertex;
use glium::winit::keyboard::Key;
use opencv::prelude::*;

#[derive(Copy, Clone)]
pub struct SkullVertex {
    position: [f32; 2],
    uv: [f32; 2],
    rotation: f32,
    state: u8,
    blend_value: f32,
    texture_id: u8,
}
implement_vertex!(
    SkullVertex,
    position,
    uv,
    rotation,
    state,
    blend_value,
    texture_id
);

struct SkullData {
    skull_vb: VertexBuffer<SkullVertex>,
    skull_idxb: IndexBuffer<u16>,
    skull_program: glium::Program,
    skull_counter: f32,
    skulls: Vec<Skull>,
}

pub struct SkullGame {
    //2nd rendering path
    skulls: Option<SkullData>,
    settings: SkullSettings,
}
impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings = load_config(config_path)?;
        Ok(SkullGame {
            skulls: None,
            settings,
        })
    }
}
impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        _config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _image: &Mat,
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
