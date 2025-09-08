use crate::PlatoConfig;
use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::skull::{Skull, SkullSpawner, SkullState};
use crate::types::game_types::GameTrait;

use ::glium::{IndexBuffer, VertexBuffer};
use glium::winit::keyboard::Key;
use glium::{Surface, implement_vertex};
use opencv::prelude::*;

#[derive(Copy, Clone)]
pub struct SkullVertex {
    position: [f32; 2],
    uv: [f32; 2],
    rotation: f32,
    state: u32,
    blend_value: f32,
    texture_id: u32,
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

fn skull_state_to_id(state: &SkullState) -> u32 {
    match state {
        SkullState::Incomming => 0,
        SkullState::Hitable => 1,
        SkullState::Killed => 2,
        SkullState::Survived => 3,
        SkullState::ToRemove => 4,
    }
}

fn create_skull_vertex_buffer(
    skull_vb: &mut glium::VertexBuffer<SkullVertex>,
    skulls: &Vec<Skull>,
    index_buffer_data: &mut Vec<u16>,
) {
    for (i, (skull, vb_entry)) in skulls.iter().zip(skull_vb.map().chunks_mut(4)).enumerate() {
        let radius = skull.scale / 2_f32;
        let blend = (skull.scale / skull.hitable_from).clamp(0_f32, 1_f32);
        let state_id = skull_state_to_id(&skull.state);
        println!("{:?}", state_id);
        vb_entry[0].position[0] = skull.center.0 - radius;
        vb_entry[0].position[1] = skull.center.1 + radius;
        vb_entry[0].uv[0] = 0_f32;
        vb_entry[0].uv[1] = 0_f32;
        vb_entry[0].rotation = skull.rotation;
        vb_entry[0].blend_value = blend;
        vb_entry[0].texture_id = 0;
        vb_entry[0].state = state_id;

        vb_entry[1].position[0] = skull.center.0 + radius;
        vb_entry[1].position[1] = skull.center.1 + radius;
        vb_entry[1].uv[0] = 1_f32;
        vb_entry[1].uv[1] = 0_f32;
        vb_entry[1].rotation = skull.rotation;
        vb_entry[1].blend_value = blend;
        vb_entry[1].texture_id = 0;
        vb_entry[1].state = state_id;

        vb_entry[2].position[0] = skull.center.0 - radius;
        vb_entry[2].position[1] = skull.center.1 - radius;
        vb_entry[2].uv[0] = 0_f32;
        vb_entry[2].uv[1] = 1_f32;
        vb_entry[2].rotation = skull.rotation;
        vb_entry[2].blend_value = blend;
        vb_entry[2].texture_id = 0;
        vb_entry[2].state = state_id;

        vb_entry[3].position[0] = skull.center.0 + radius;
        vb_entry[3].position[1] = skull.center.1 - radius;
        vb_entry[3].uv[0] = 1_f32;
        vb_entry[3].uv[1] = 1_f32;
        vb_entry[3].rotation = skull.rotation;
        vb_entry[3].blend_value = blend;
        vb_entry[3].texture_id = 0;
        vb_entry[3].state = state_id;

        let num = i as u16;
        index_buffer_data.push(num * 4);
        index_buffer_data.push(num * 4 + 1);
        index_buffer_data.push(num * 4 + 2);
        index_buffer_data.push(num * 4 + 1);
        index_buffer_data.push(num * 4 + 3);
        index_buffer_data.push(num * 4 + 2);
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
                    skull.update(&self.mask, timestep)?;
                    match skull.state {
                        SkullState::Killed => {
                            //skull.state = SkullState::ToRemove;
                            self.game_state.current_score += 1_f32;
                            //spawn particles and shit
                        }
                        SkullState::Survived => {
                            //skull.state = SkullState::ToRemove;
                            self.game_state.current_score -= 1_f32;
                            //spawn particles and shit
                        }
                        _ => {}
                    };
                }

                self.skull_spawner.maybe_spawn(&mut data.skulls, &timestep);
                Ok(())
            }
            None => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }?;
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
