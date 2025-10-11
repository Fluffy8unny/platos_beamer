use opencv::core::{find_non_zero, Point};
use opencv::prelude::*;

use crate::game::skull_game::particle::{update_gravity_particle, Particle, Target};
fn convert_opencv_to_opengl_coords(pos: i32, dim: i32) -> f32 {
    let rel_pos = (pos as f32) / (dim as f32); //[0,1]
    rel_pos * 2_f32 - 1_f32 //[-1,1]
}
fn convert_point_opencv_to_opengl(pt: Point, dims: (i32, i32)) -> (f32, f32) {
    let x = convert_opencv_to_opengl_coords(pt.x, dims.0);
    let y = convert_opencv_to_opengl_coords(pt.y, dims.1);
    (x, -y)
}
pub fn spawn_based_on_mask(mask: &Mat) -> Result<Vec<Particle>, Box<dyn std::error::Error>> {
    let dims = (mask.rows(), mask.cols());
    let mut positions = Mat::default();
    find_non_zero(&mask, &mut positions)?;

    let particle_vector: Vec<Particle> = (0_i32..positions.rows())
        .map(|i| -> Result<Particle, Box<dyn std::error::Error>> {
            let pos = positions.at::<Point>(i)?;
            let gl_pos = convert_point_opencv_to_opengl(*pos, dims);
            let target = Target {
                center: (gl_pos.0, 1.0),
                gravity: 1.0,
                size: 0.1,
            };

            Ok(Particle::new(
                gl_pos,
                0.01,
                (0.9, 0.9, 0.9),
                (0.0, 1.0),
                target,
                update_gravity_particle,
            ))
        })
        .filter_map(|res| res.ok())
        .collect();
    Ok(particle_vector)
}
