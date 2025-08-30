use crate::config::{ShaderConfig, load_config};
use crate::display::display_window::DisplayType;
use opencv::core::flip;
use opencv::prelude::*;
pub fn load_shaders(
    path: &str,
    display: &DisplayType,
) -> Result<glium::Program, Box<dyn std::error::Error>> {
    let shaders: ShaderConfig = load_config(path)?;
    Ok(glium::Program::from_source(
        display,
        &shaders.vertex,
        &shaders.fragment,
        None,
    )?)
}

pub fn mat_1c_to_texture_r(
    display: &DisplayType,
    mat: &Mat,
) -> Result<glium::Texture2d, Box<dyn std::error::Error>> {
    //we need to flip the image, because opengl is fucking retarded
    let data: Vec<u8> = mat.data_bytes()?.iter().rev().copied().collect();
    let text2d = glium::texture::RawImage2d {
        data: std::borrow::Cow::from(data),
        width: mat.cols() as u32,
        height: mat.rows() as u32,
        format: glium::texture::ClientFormat::U8,
    };

    Ok(glium::Texture2d::new(display, text2d)?)
}
