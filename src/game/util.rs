use crate::config::{load_config, ShaderConfig};
use crate::display::display_window::DisplayType;
use image::ImageReader;
use opencv::imgproc::{cvt_color, COLOR_BGR2GRAY};
use opencv::prelude::*;

pub fn load_shaders(
    path: &str,
    display: &DisplayType,
) -> Result<glium::Program, Box<dyn std::error::Error>> {
    let shaders: ShaderConfig = load_config(path)?;
    println!("loading {:?}", shaders.name);
    Ok(glium::Program::from_source(
        display,
        &shaders.vertex,
        &shaders.fragment,
        None,
    )?)
}

pub fn load_rgb_image_as_texture(
    path: &str,
    display: &DisplayType,
) -> Result<glium::Texture2d, Box<dyn std::error::Error>> {
    let image = ImageReader::open(path)?.decode()?.to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    Ok(glium::texture::Texture2d::new(display, image)?)
}

pub fn image_to_gray_texture_r(
    display: &DisplayType,
    mat: &Mat,
) -> Result<glium::Texture2d, Box<dyn std::error::Error>> {
    let mut gray_img = Mat::default();
    match mat.channels() {
        1 => {
            gray_img = mat.clone();
        }
        3 => {
            cvt_color(
                &mat,
                &mut gray_img,
                COLOR_BGR2GRAY,
                1,
                opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
            )?;
        }
        _ => {
            return Err("invalid matrix provided. Input needs to be either gray or BGR".into());
        }
    }
    mat_1c_to_texture_r(display, &gray_img)
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
