use opencv::core::{Point, find_non_zero};
use opencv::prelude::*;
use rand::seq::IndexedRandom;
use rand::{Rng, rng};

use crate::game::skull_game::particle::{Particle, Target, update_linear_particle};
fn convert_opencv_to_opengl_coords(pos: i32, dim: i32) -> f32 {
    let rel_pos = (pos as f32) / (dim as f32); //[0,1]
    2_f32 * rel_pos - 1.0_f32
}
fn convert_point_opencv_to_opengl(pt: Point, dims: (i32, i32)) -> (f32, f32) {
    let x = convert_opencv_to_opengl_coords(pt.y, dims.0);
    let y = convert_opencv_to_opengl_coords(pt.x, dims.1);
    (-y, -x)
}
pub fn spawn_based_on_mask(
    mask: &Mat,
    max_particles: usize,
) -> Result<Vec<Particle>, Box<dyn std::error::Error>> {
    let dims = (mask.rows(), mask.cols());
    let mut positions = Mat::default();
    find_non_zero(&mask, &mut positions)?;

    let mut randomizer = rng();
    let particle_vector: Vec<Particle> = (0_i32..positions.rows())
        .map(|i| -> Result<Particle, Box<dyn std::error::Error>> {
            let pos = positions.at::<Point>(i)?;
            let gl_pos = convert_point_opencv_to_opengl(*pos, dims);
            let target = Target {
                center: (gl_pos.0, 1.0),
                gravity: 1.0,
                size: 0.1,
            };

            let v = randomizer.random_range(3.7_f32..9.3_f32);
            let s = randomizer.random_range(0.00125_f32..0.02_f32);
            let x = randomizer.random_range(-1_f32..1_f32);

            let o = 1.0 - x.abs();
            println!("{:?}", o);
            Ok(Particle::new(
                (gl_pos.0, -1.0),
                s,
                (0.0, o, o),
                o,
                (x, v),
                target,
                update_linear_particle,
            ))
        })
        .filter_map(|res| res.ok())
        .collect();
    let max_number = particle_vector.len().min(max_particles);
    let selected_particles: Vec<Particle> = particle_vector
        .choose_multiple(&mut rand::rng(), max_number)
        .cloned()
        .collect();
    Ok(selected_particles)
}
