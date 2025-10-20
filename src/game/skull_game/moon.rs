use ::glium::{IndexBuffer, VertexBuffer};
use glium::implement_vertex;

use crate::display::timestep::TimeStep;
use crate::{
    display::display_window::DisplayType, game::skull_game::util::generate_index_for_quad,
};

pub enum MoonState {
    Alive,
    Dead,
}
pub struct Moon {
    pub position: (f32, f32),
    pub scale: f32,
    pub life: u32,
    pub max_life: u32,
    pub state: MoonState,
    timer: TimeStep,
}

#[derive(Copy, Clone)]
pub struct MoonVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub blend_value: f32,
    pub texture_id: f32,
}
implement_vertex!(MoonVertex, position, uv, texture_id, blend_value,);

pub fn create_moon_vertex_buffer(
    moon: &Moon,
    display: &DisplayType,
) -> Result<(glium::VertexBuffer<MoonVertex>, glium::IndexBuffer<u32>), Box<dyn std::error::Error>>
{
    let mut moon_vertex_buffer: glium::VertexBuffer<MoonVertex> =
        glium::VertexBuffer::empty_dynamic(display, 4)?;
    let mut moon_ib: Vec<u32> = Vec::with_capacity(4);
    let blend_value = moon.life as f32 / moon.max_life as f32;
    {
        let moon_vb = &mut moon_vertex_buffer.map();

        moon_vb[0].position[0] = moon.position.0 - moon.scale;
        moon_vb[0].position[1] = moon.position.1 + moon.scale;
        moon_vb[0].uv[0] = 0_f32;
        moon_vb[0].uv[1] = 0_f32;

        moon_vb[1].position[0] = moon.position.0 + moon.scale;
        moon_vb[1].position[1] = moon.position.1 + moon.scale;
        moon_vb[1].uv[0] = 1_f32;
        moon_vb[1].uv[1] = 0_f32;

        moon_vb[2].position[0] = moon.position.0 - moon.scale;
        moon_vb[2].position[1] = moon.position.1 - moon.scale;
        moon_vb[2].uv[0] = 0_f32;
        moon_vb[2].uv[1] = 1_f32;

        moon_vb[3].position[0] = moon.position.0 + moon.scale;
        moon_vb[3].position[1] = moon.position.1 - moon.scale;
        moon_vb[3].uv[0] = 1_f32;
        moon_vb[3].uv[1] = 1_f32;

        for i in 0..4 {
            moon_vb[i].texture_id = 0_f32;
            moon_vb[i].blend_value = blend_value;
        }
    }
    generate_index_for_quad(0, &mut moon_ib);
    let moon_idxb: glium::IndexBuffer<u32> = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &moon_ib,
    )?;
    Ok((moon_vertex_buffer, moon_idxb))
}

impl Moon {
    pub fn new(starting_life: u32, position: (f32, f32), scale: f32) -> Self {
        Moon {
            life: starting_life,
            max_life: starting_life,
            state: MoonState::Alive,
            position,
            scale,
            timer: TimeStep::new(),
        }
    }

    pub fn get_time(&mut self) -> f32 {
        self.timer.update();
        self.timer.runtime
    }

    pub fn hit(&mut self, damage: u32) {
        self.life = self.life.saturating_sub(damage);
        if self.life == 0 {
            self.state = MoonState::Dead
        };
    }
    pub fn heal(&mut self, healing: u32) {
        self.life = self.life.saturating_add(healing).clamp(0_u32, self.max_life);
    }
}

pub struct MoonData {
    pub moon_vb: VertexBuffer<MoonVertex>,
    pub moon_idxb: IndexBuffer<u32>,
    pub moon: Moon,
}
