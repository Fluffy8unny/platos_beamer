use crate::display::display_window::DisplayType;

use glium::DrawParameters;
use glium::texture::Texture2dArray;

use opencv::core::Vec3b;
use opencv::imgcodecs::{ImreadModes, imread};
use opencv::imgproc::{COLOR_BGR2GRAY, cvt_color};
use opencv::{Result, prelude::*};

use rand::{Rng, rng};

pub fn generate_index_for_quad(counter: usize, index_buffer_data: &mut Vec<u32>) {
    let num = counter as u32;
    index_buffer_data.push(num * 4);
    index_buffer_data.push(num * 4 + 1);
    index_buffer_data.push(num * 4 + 2);
    index_buffer_data.push(num * 4 + 1);
    index_buffer_data.push(num * 4 + 3);
    index_buffer_data.push(num * 4 + 2);
}

pub fn get_boxed_opencv_error(name: &str, code: i32) -> Box<opencv::Error> {
    Box::new(opencv::Error {
        message: format!("{} data was not initialized", name).to_string(),
        code,
    })
}

pub fn get_random_sound_name(base_name: &str, number_of_sounds: u32) -> String {
    let mut randomizer = rng();
    let sound_index = randomizer.random_range(1..=number_of_sounds);
    format!("{}_{}", base_name, sound_index)
}

pub fn get_draw_params() -> DrawParameters<'static> {
    glium::DrawParameters {
        blend: glium::draw_parameters::Blend::alpha_blending(),
        ..Default::default()
    }
}

pub fn load_texture(
    texture_paths: &Vec<String>,
    mask_color: (u8, u8, u8),
    display: &DisplayType,
) -> Result<Texture2dArray, Box<dyn std::error::Error>> {
    let texture_data = load_texture_data(texture_paths, mask_color)?;
    Ok(Texture2dArray::new(display, texture_data)?)
}

fn load_texture_data(
    texture_paths: &Vec<String>,
    mask_color: (u8, u8, u8),
) -> Result<Vec<Vec<Vec<f32>>>, Box<dyn std::error::Error>> {
    let mut tex_data = Vec::default();
    //todo change to map expression somehow
    for path in texture_paths {
        let img = imread(path, ImreadModes::IMREAD_COLOR_BGR.into())?;
        let mut gray = Mat::default();
        cvt_color(
            &img,
            &mut gray,
            COLOR_BGR2GRAY,
            1,
            opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let num_bytes: usize = (img.cols() * img.rows()).try_into()?;
        let mut float_data: Vec<f32> = Vec::with_capacity(num_bytes);
        let mask_vec = Vec3b::from_array(mask_color.into());
        for (pixel, gray_pixel) in img
            .data_typed::<Vec3b>()?
            .iter()
            .zip(gray.data_typed::<u8>()?)
        {
            let lum_value = (*gray_pixel as f32) / 255_f32;
            float_data.push(if *pixel == mask_vec { 0_f32 } else { lum_value });
        }
        let img_data: Vec<Vec<f32>> = float_data
            .chunks(img.rows().try_into()?)
            .map(|x| x.into())
            .collect();
        tex_data.push(img_data);
    }
    Ok(tex_data)
}
