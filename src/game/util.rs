use crate::config::{ShaderConfig, load_config};
use crate::display::display_window::DisplayType;
use image::ImageReader;
use opencv::imgproc::{COLOR_BGR2GRAY, cvt_color};
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

#[derive(Copy, Clone)]
pub struct Interpolator<T> {
    pub target_value: T,
    pub current_value: T,
    interp_speed: T,
}

impl<T> Interpolator<T> {
    pub fn new(starting_value: T, interp_speed: T) -> Self
    where
        T: Copy,
    {
        Self {
            target_value: starting_value,
            current_value: starting_value,
            interp_speed,
        }
    }

    pub fn reset(&mut self, value: T)
    where
        T: Copy,
    {
        self.target_value = value;
        self.current_value = value;
    }

    pub fn change_target(&mut self, new_target: T)
    where
        T: Copy,
    {
        self.target_value = new_target;
    }

    pub fn update(&mut self)
    where
        T: Copy
            + std::ops::Sub<Output = T>
            + std::fmt::Debug
            + std::ops::Neg<Output = T>
            + std::ops::AddAssign
            + std::ops::Mul<Output = T>
            + Default
            + PartialOrd,
    {
        if self.current_value == self.target_value {
            return;
        }
        let diff = self.target_value - self.current_value;
        let step = if diff > T::default() {
            self.interp_speed
        } else {
            -self.interp_speed
        };
        if step * step > diff * diff {
            self.current_value = self.target_value;
            return;
        }
        self.current_value += step;
    }
}
