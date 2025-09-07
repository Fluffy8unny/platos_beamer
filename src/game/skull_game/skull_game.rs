use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::skull::{self, Skull, SkullState, hit_test, update};
use crate::types::game_types::GameTrait;
use crate::{PlatoConfig, display};

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
fn skull_state_to_id(state: &SkullState) -> u32 {
    match state {
        SkullState::Incomming => 0,
        SkullState::Hitable => 1,
        SkullState::Killed => 2,
        SkullState::Survived => 3,
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
        let test_skull = Skull {
            center: (0_f32, 0_f32),
            scale: 0.5_f32,
            rotation: 0_f32,
            state: SkullState::Hitable,
            hitable_from: 0.2,
            max_scale: 0.5,
            direction: (0_f32, 1_f32),
            speed: 0.01,
            threshold: 0.01,
        };
        self.skulls = Some(update_skull_state(vec![test_skull], display)?);
        Ok(())
    }

    fn update(
        &mut self,
        _image: &Mat,
        mask: &Mat,
        _display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.skulls {
            Some(data) => {
                for skull in data.skulls.iter_mut().filter(|skull| {
                    if let SkullState::Hitable = skull.state {
                        true
                    } else {
                        false
                    }
                }) {
                    if hit_test(&skull, mask)? {
                        skull.state = SkullState::Killed;
                        //spawn particles and stuff
                    }
                }
                Ok(())
            }
            None => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        _timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //draw skulls
        match &self.skulls {
            Some(skulls) => frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                &skulls.skull_program,
                &glium::uniforms::EmptyUniforms,
                &glium::DrawParameters::default(),
            )?,
            None => println!("skull shader not initialized"),
        };
        Ok(())
    }

    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
