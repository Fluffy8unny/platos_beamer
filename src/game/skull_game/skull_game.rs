use crate::PlatoConfig;
use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::skull::{Skull, SkullSpawner, SkullState, create_skull_vertex_buffer};
use crate::types::game_types::GameTrait;

use ::glium::{IndexBuffer, VertexBuffer};
use glium::winit::keyboard::Key;
use glium::{Surface, implement_vertex};
use opencv::prelude::*;

#[derive(Copy, Clone)]
pub struct SkullVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub rotation: f32,
    pub state: u32,
    pub blend_value: f32,
    pub texture_id: u32,
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
    skulls: Vec<Skull>,
}

pub enum GameEvent {
    Killed { pos: (f32, f32), scale: f32 },
    Escaped { pos: (f32, f32), scale: f32 },
}

struct GameState {
    current_score: f32,
}

pub struct SkullGame {
    //2nd rendering path
    skull_data: Option<SkullData>,
    skull_spawner: SkullSpawner,
    mask: Option<Mat>,
    settings: SkullSettings,
    game_state: GameState,
}

impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings: SkullSettings = load_config(config_path)?;
        Ok(SkullGame {
            skull_data: None,
            skull_spawner: SkullSpawner {
                time_since: 0_f32,
                settings: settings.clone(),
            },
            mask: None,
            settings,
            game_state: GameState {
                current_score: 0_f32,
            },
        })
    }
}

fn update_skull_state(
    skulls: Vec<Skull>,
    display: &DisplayType,
) -> Result<SkullData, Box<dyn std::error::Error>> {
    let skull_count = skulls.len();

    let mut skull_vb: glium::VertexBuffer<SkullVertex> =
        glium::VertexBuffer::empty_dynamic(display, skull_count * 4)?;
    let mut index_buffer_data: Vec<u16> = Vec::with_capacity(skull_count * 6);
    //we can't map over a Vertex buffer length 0
    if skull_count > 0 {
        create_skull_vertex_buffer(&mut skull_vb, &skulls, &mut index_buffer_data);
    }

    let skull_program = load_shaders("src/shaders/skull_game_skull.toml", display)?;
    let skull_idxb: glium::IndexBuffer<u16> = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &index_buffer_data,
    )?;

    Ok(SkullData {
        skull_vb,
        skull_idxb,
        skull_program,
        skulls,
    })
}

impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        _config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.skull_data = Some(update_skull_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);
        Ok(())
    }

    fn update(
        &mut self,
        _image: &Mat,
        mask: &Mat,
        _display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mask = Some(mask.clone());
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        display: &DisplayType,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //update skulls
        match &mut self.skull_data {
            Some(data) => {
                for skull in data.skulls.iter_mut() {
                    match skull.update(&self.mask, timestep)? {
                        Some(GameEvent::Killed { pos, scale }) => {}
                        Some(GameEvent::Escaped { pos, scale }) => {}
                        None => {}
                    }
                }
                let _ = data
                    .skulls
                    .retain(|skull| !matches!(skull.state, SkullState::ToRemove));
                self.skull_spawner.maybe_spawn(&mut data.skulls, &timestep);
                Ok(())
            }
            None => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }?;

        //update buffer data
        self.skull_data = Some(update_skull_state(
            self.skull_data.as_ref().unwrap().skulls.clone(),
            display,
        )?);

        //draw skulls
        match &self.skull_data {
            Some(skulls) => Ok(frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                &skulls.skull_program,
                &glium::uniforms::EmptyUniforms,
                &glium::DrawParameters::default(),
            )?),
            None => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }
    }
    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
