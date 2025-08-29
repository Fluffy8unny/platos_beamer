use std::error::Error;

use crate::{
    config::{PlatoConfig, ShaderConfig, load_config},
    display::{
        display_window::DisplayType,
        primitves::{QUAD_INDICES, Vertex, get_quad_buffer},
    },
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
    pub fn draw(&self, frame: &mut glium::Frame) -> Result<(), Box<dyn std::error::Error>> {
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
        img: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let texture = get_texture_from_image(img, display)?;
        self.texture = texture;
        Ok(())
    }
}

fn get_empty_minimap() -> Mat {
    Mat::zeros(640, 480, CV_8UC1).unwrap().to_mat().unwrap()
}

fn get_texture_from_image(
    img: &Mat,
    display: &DisplayType,
) -> Result<glium::Texture2d, Box<dyn Error>> {
    let text2d = glium::texture::RawImage2d {
        data: std::borrow::Cow::from(img.data_bytes()?),
        width: img.cols() as u32,
        height: img.rows() as u32,
        format: glium::texture::ClientFormat::U8,
    };
    Ok(glium::Texture2d::new(display, text2d)?)
}

fn load_shaders(display: &DisplayType) -> Result<glium::Program, Box<dyn std::error::Error>> {
    let shaders: ShaderConfig = load_config("src/shaders/minimap.toml")?;
    Ok(glium::Program::from_source(
        display,
        &shaders.vertex,
        &shaders.fragment,
        None,
    )?)
}

fn get_buffers(
    display: &DisplayType,
    pos: (f32, f32),
    dims: (f32, f32),
) -> Result<BufferCollection, Box<dyn std::error::Error>> {
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

pub fn create_minimap(
    display: &DisplayType,
    config: &PlatoConfig,
) -> Result<Minimap, Box<dyn std::error::Error>> {
    let default_texture = get_texture_from_image(&get_empty_minimap(), display)?;
    Ok(Minimap {
        buffers: get_buffers(
            display,
            config.minimap_config.position,
            config.minimap_config.dims,
        )?,
        texture: default_texture,
        program: load_shaders(display)?,
    })
}
