use std::error::Error;

use crate::{
    config::{ShaderConfig, load_config},
    display::{
        display_window::DisplayType,
        primitves::{QUAD_INDICES, Vertex, get_quad_buffer},
    },
};
use glium;
use glium::Surface;
use glium::uniform;
use opencv::imgproc::cvt_color;
use opencv::prelude::*;
use opencv::{core::CV_8UC1, imgproc::COLOR_GRAY2BGR};
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
    let dims = (img.rows() as u32, img.cols() as u32);
    println!("{:?}", img);
    //let mut rgb_mat= Mat::default();
    //cvt_color(img,&mut rgb_mat, COLOR_GRAY2BGR, 3, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)?;
    //let data_bytes = rgb_mat.data_bytes()?;
    let text2d = glium::texture::RawImage2d {
        data: std::borrow::Cow::from(img.data_bytes()?),
        width: dims.1,
        height: dims.0,
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

fn get_buffers(display: &DisplayType) -> Result<BufferCollection, Box<dyn std::error::Error>> {
    let verticies = get_quad_buffer((0.5, 1.0), (0.5, 0.8));
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

pub fn create_minimap(display: &DisplayType) -> Result<Minimap, Box<dyn std::error::Error>> {
    let default_texture = get_texture_from_image(&get_empty_minimap(), display)?;
    Ok(Minimap {
        buffers: get_buffers(display)?,
        texture: default_texture,
        program: load_shaders(display)?,
    })
}
