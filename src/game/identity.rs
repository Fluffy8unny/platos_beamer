use crate::PlatoConfig;
use crate::display::display_window::DisplayType;
use crate::display::primitves::{QUAD_INDICES, Vertex, get_quad_buffer};
use crate::display::timestep::TimeStep;
use crate::game::util::{load_shaders, mat_1c_to_texture_r};
use crate::types::GameTrait;

use glium::Surface;
use glium::uniform;
use glium::winit::keyboard::Key;
use opencv::prelude::*;

pub struct IdentityGame {
    current_mask: Option<glium::Texture2d>,
    program: Option<glium::Program>,
    vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
    index_buffer: Option<glium::IndexBuffer<u16>>,
}

impl IdentityGame {
    pub fn new() -> IdentityGame {
        IdentityGame {
            current_mask: None,
            program: None,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

impl GameTrait for IdentityGame {
    fn init(
        &mut self,
        display: &DisplayType,
        _config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let verticies = get_quad_buffer((-1_f32, 1_f32), (-1_f32, 1_f32));
        let vertex_buffer = glium::VertexBuffer::new(display, &verticies)?;
        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &QUAD_INDICES,
        )?;
        let program = load_shaders("src/shaders/minimap.toml", display)?;

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.program = Some(program);
        Ok(())
    }

    fn update(
        &mut self,
        _image: &Mat,
        mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.current_mask = Some(mat_1c_to_texture_r(display, mask)?);
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        _display: &DisplayType,
        _timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        frame.draw(
            self.vertex_buffer.as_ref().ok_or("no vertext buffer")?,
            self.index_buffer.as_ref().ok_or("no index buffer")?,
            self.program.as_ref().ok_or("no program")?,
            &uniform! { tex: self.current_mask.as_ref().ok_or("no texture")?},
            &glium::DrawParameters::default(),
        )?;
        Ok(())
    }

    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
