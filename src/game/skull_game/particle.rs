use std::f32::consts::PI;

use glium::implement_vertex;
use rand::{Rng, rng};

use crate::display::timestep::TimeStep;
use crate::game::skull_game::util::generate_index_for_quad;

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub center: (f32, f32),
    pub gravity: f32,
    pub size: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ParticleState {
    Alive,
    ToRemove,
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    color: (f32, f32, f32),
    center: (f32, f32),
    update_function: fn(&mut Particle),
    target: Target,
    scale: f32,
    velocity: (f32, f32),
    timer: TimeStep,
    pub state: ParticleState,
}
#[derive(Copy, Clone)]
pub struct ParticleVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 3],
    pub blend_value: f32,
}

implement_vertex!(ParticleVertex, position, uv, color, blend_value,);

pub fn create_particle_vertex_buffer(
    skull_vb: &mut glium::VertexBuffer<ParticleVertex>,
    skulls: &Vec<Particle>,
    index_buffer_data: &mut Vec<u16>,
) {
    for (i, (particle, vb_entry)) in skulls.iter().zip(skull_vb.map().chunks_mut(4)).enumerate() {
        let radius = particle.scale / 2_f32;

        vb_entry[0].position[0] = particle.center.0 - radius;
        vb_entry[0].position[1] = particle.center.1 + radius;
        vb_entry[0].uv[0] = 0_f32;
        vb_entry[0].uv[1] = 0_f32;
        vb_entry[0].color = [particle.color.0, particle.color.1, particle.color.2];
        vb_entry[0].blend_value = 1_f32;

        vb_entry[1].position[0] = particle.center.0 + radius;
        vb_entry[1].position[1] = particle.center.1 + radius;
        vb_entry[1].uv[0] = 1_f32;
        vb_entry[1].uv[1] = 0_f32;
        vb_entry[1].color = [particle.color.0, particle.color.1, particle.color.2];
        vb_entry[1].blend_value = 1_f32;

        vb_entry[2].position[0] = particle.center.0 - radius;
        vb_entry[2].position[1] = particle.center.1 - radius;
        vb_entry[2].uv[0] = 0_f32;
        vb_entry[2].uv[1] = 1_f32;
        vb_entry[2].color = [particle.color.0, particle.color.1, particle.color.2];
        vb_entry[2].blend_value = 1_f32;

        vb_entry[3].position[0] = particle.center.0 + radius;
        vb_entry[3].position[1] = particle.center.1 - radius;
        vb_entry[3].uv[0] = 1_f32;
        vb_entry[3].uv[1] = 1_f32;
        vb_entry[3].color = [particle.color.0, particle.color.1, particle.color.2];
        vb_entry[3].blend_value = 1_f32;

        generate_index_for_quad(i, index_buffer_data);
    }
}

pub fn update_gravity_particle(particle: &mut Particle) {
    let dt = particle.timer.time_delta / 1000_f32;
    let dx = (
        particle.target.center.0 - particle.center.0,
        particle.target.center.1 - particle.center.1,
    );
    let magnitude_dx = (dx.0 * dx.0 + dx.1 * dx.1).sqrt();
    if magnitude_dx < 0.01 {
        particle.state = ParticleState::ToRemove;
    }

    let dv = (
        dx.0 * particle.target.gravity / magnitude_dx,
        dx.1 * particle.target.gravity / magnitude_dx,
    );

    particle.velocity.0 = particle.velocity.0 * 0.95 + dv.0 * dt;
    particle.velocity.1 = particle.velocity.1 * 0.95 + dv.1 * dt;

    particle.center.0 += particle.velocity.0 * dt;
    particle.center.1 += particle.velocity.1 * dt;
}

pub fn generate_random_particles_around_point(
    point: (f32, f32),
    area: f32,
    target: Target,
    max_initial_speed: f32,
    color: (f32, f32, f32),
    scale: f32,
    number: usize,
) -> Vec<Particle> {
    let mut result: Vec<Particle> = Vec::with_capacity(number);
    let mut randomizer = rng();

    for _ in 0..number {
        let r = randomizer.random_range(0_f32..area / 2_f32);
        let phi = randomizer.random_range(0_f32..2_f32 * PI);

        let x = phi.cos() * r + point.0;
        let y = phi.sin() * r + point.1;
        let v_0 = (x - point.0, y - point.1);
        let v_norm = max_initial_speed / (v_0.0 * v_0.0 + v_0.1 * v_0.1).sqrt();
        let particle = Particle::new(
            (x, y),
            scale,
            color,
            (v_0.0 * v_norm, v_0.1 * v_norm),
            target,
            update_gravity_particle,
        );
        result.push(particle);
    }
    result
}

impl Particle {
    pub fn new(
        center: (f32, f32),
        scale: f32,
        color: (f32, f32, f32),
        velocity: (f32, f32),
        target: Target,
        update_function: fn(&mut Particle),
    ) -> Particle {
        Particle {
            color,
            center,
            target,
            scale,
            velocity,
            timer: TimeStep::new(),
            state: ParticleState::Alive,
            update_function,
        }
    }

    pub fn update(&mut self) {
        self.timer.update();
        (self.update_function)(self);
    }
}
