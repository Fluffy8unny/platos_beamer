use ::glium::{IndexBuffer, VertexBuffer};
use glium::implement_vertex;

use crate::display::timestep::TimeStep;
use crate::game::skull_game::{moon, position_visualization};
use crate::game::util::Interpolator;
use crate::{
    display::display_window::DisplayType, game::skull_game::util::generate_index_for_quad,
};

#[derive(Copy, Clone)]
pub enum MoonState {
    Alive,
    Dead,
}

#[derive(Copy, Clone)]
pub struct Moon {
    pub position: (f32, f32),
    pub max_position: (f32, f32),
    pub current_position: (f32, f32),
    pub scale: f32,
    pub life: Interpolator<f32>,
    pub max_life: u32,
    pub state: MoonState,
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
    let blend_value = moon.get_life_fraction();
    let position = moon.current_position;
    {
        let moon_vb = &mut moon_vertex_buffer.map();

        moon_vb[0].position[0] = position.0 - moon.scale;
        moon_vb[0].position[1] = position.1 + moon.scale;
        moon_vb[0].uv[0] = 0_f32;
        moon_vb[0].uv[1] = 0_f32;

        moon_vb[1].position[0] = position.0 + moon.scale;
        moon_vb[1].position[1] = position.1 + moon.scale;
        moon_vb[1].uv[0] = 1_f32;
        moon_vb[1].uv[1] = 0_f32;

        moon_vb[2].position[0] = position.0 - moon.scale;
        moon_vb[2].position[1] = position.1 - moon.scale;
        moon_vb[2].uv[0] = 0_f32;
        moon_vb[2].uv[1] = 1_f32;

        moon_vb[3].position[0] = position.0 + moon.scale;
        moon_vb[3].position[1] = position.1 - moon.scale;
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
    pub fn new(settings: crate::game::skull_game::config::MoonSettings) -> Self {
        Moon {
            life: Interpolator::new(settings.starting_life as f32, 0.01),
            max_life: settings.starting_life,
            state: MoonState::Alive,
            position: settings.position,
            max_position: settings.max_position,
            current_position: settings.position,
            scale: settings.scale,
        }
    }

    pub fn get_position(&self) -> (f32, f32) {
        let blend_value = 1.0 - self.get_life_fraction();
        let dpos = (
            self.max_position.0 - self.position.0,
            self.max_position.1 - self.position.1,
        );
        (
            blend_value * dpos.0 + self.position.0,
            blend_value * dpos.1 + self.position.1,
        )
    }

    pub fn get_life_fraction(&self) -> f32 {
        self.life.current_value / self.max_life as f32
    }

    pub fn update_position(&mut self) {
        self.current_position = self.get_position();
    }

    pub fn hit(&mut self, damage: u32) {
        let updated_life = (self.life.target_value - damage as f32).max(0_f32);
        self.life.change_target(updated_life);
        println!("got hit {:?}", self.life.current_value);
        if self.life.current_value == 0_f32 {
            self.state = MoonState::Dead
        };
    }
    pub fn heal(&mut self, healing: u32) {
        let updated_life = (self.life.target_value + healing as f32).min(self.max_life as f32);
        self.life.change_target(updated_life);
        println!("got healed {:?}", self.life.current_value);
    }
}

pub fn update_moon_data(
    moon_data: &MoonData,
    display: &DisplayType,
) -> Result<MoonData, Box<dyn std::error::Error>> {
    let (moon_vb, moon_idxb) = create_moon_vertex_buffer(&moon_data.moon, display)?;

    Ok(MoonData {
        moon_vb,
        moon_idxb,
        moon: moon_data.moon,
    })
}

pub struct MoonData {
    pub moon_vb: VertexBuffer<MoonVertex>,
    pub moon_idxb: IndexBuffer<u32>,
    pub moon: Moon,
}
