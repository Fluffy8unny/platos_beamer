use std::error::Error;

use crate::{
    config::PlatoConfig,
    display::{
        display_window::DisplayType,
        primitves::{QUAD_INDICES, Vertex, get_quad_buffer},
    },
    game::util::{load_shaders, mat_1c_to_texture_r},
};

use glium::Surface;
use glium::uniform;
use opencv::core::CV_8UC1;
use opencv::prelude::*;

pub struct BufferCollection {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
}

pub struct Minimap {
    buffers: BufferCollection,
    texture: glium::Texture2d,
    program: glium::Program,
}

impl Minimap {
    pub fn new(display: &DisplayType, config: &PlatoConfig) -> Result<Minimap, Box<dyn Error>> {
        let default_texture = mat_1c_to_texture_r(display, &get_empty_minimap())?;
        Ok(Minimap {
            buffers: get_buffers(
                display,
                config.minimap_config.position,
                config.minimap_config.dims,
            )?,
            texture: default_texture,
            program: load_shaders("src/shaders/minimap.toml", display)?,
        })
    }
    pub fn draw(&self, frame: &mut glium::Frame) -> Result<(), Box<dyn Error>> {
        frame.draw(
            &self.buffers.vertex_buffer,
            &self.buffers.index_buffer,
            &self.program,
            &uniform! { tex: &self.texture },
            &glium::DrawParameters::default(),
        )?;
        Ok(())
    }

    pub fn update_texture(
        &mut self,
        _image: &Mat, //not implemented yet
        mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn Error>> {
        let texture = mat_1c_to_texture_r(display, mask)?;
        self.texture = texture;
        Ok(())
    }
}

fn get_empty_minimap() -> Mat {
    Mat::zeros(640, 480, CV_8UC1).unwrap().to_mat().unwrap()
}

fn get_buffers(
    display: &DisplayType,
    pos: (f32, f32),
    dims: (f32, f32),
) -> Result<BufferCollection, Box<dyn Error>> {
    let verticies = get_quad_buffer((pos.0, pos.0 + dims.0), (pos.1, pos.1 + dims.1));
    let vertex_buffer = glium::VertexBuffer::new(display, &verticies)?;
    let index_buffer = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &QUAD_INDICES,
    )?;
    Ok(BufferCollection {
        vertex_buffer,
        index_buffer,
    })
}
