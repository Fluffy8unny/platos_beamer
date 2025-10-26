use std::f32::consts::PI;

use crate::display::display_window::DisplayType;
use ::glium::{IndexBuffer, VertexBuffer};
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
    opacity: f32,
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
    index_buffer_data: &mut Vec<u32>,
) {
    for (i, (particle, vb_entry)) in skulls.iter().zip(skull_vb.map().chunks_mut(4)).enumerate() {
        let radius = particle.scale / 2_f32;

        vb_entry[0].position[0] = particle.center.0 - radius;
        vb_entry[0].position[1] = particle.center.1 + radius;
        vb_entry[0].uv[0] = -1_f32;
        vb_entry[0].uv[1] = -1_f32;

        vb_entry[1].position[0] = particle.center.0 + radius;
        vb_entry[1].position[1] = particle.center.1 + radius;
        vb_entry[1].uv[0] = 1_f32;
        vb_entry[1].uv[1] = -1_f32;

        vb_entry[2].position[0] = particle.center.0 - radius;
        vb_entry[2].position[1] = particle.center.1 - radius;
        vb_entry[2].uv[0] = -1_f32;
        vb_entry[2].uv[1] = 1_f32;

        vb_entry[3].position[0] = particle.center.0 + radius;
        vb_entry[3].position[1] = particle.center.1 - radius;
        vb_entry[3].uv[0] = 1_f32;
        vb_entry[3].uv[1] = 1_f32;

        for entry in vb_entry.iter_mut().take(4) {
            entry.color = [particle.color.0, particle.color.1, particle.color.2];
            entry.blend_value = particle.opacity;
        }

        generate_index_for_quad(i, index_buffer_data);
    }
}

fn get_dt(particle: &Particle) -> f32 {
    particle.timer.time_delta / 1000_f32
}

fn get_distance_to_target(particle: &Particle) -> (f32, f32) {
    (
        particle.target.center.0 - particle.center.0,
        particle.target.center.1 - particle.center.1,
    )
}

fn magnitude(vector: (f32, f32)) -> f32 {
    (vector.0 * vector.0 + vector.1 * vector.1).sqrt()
}

fn update_particle_based_on_acceleration(
    particle: &mut Particle,
    dv: (f32, f32),
    dt: f32,
    drag: f32,
) {
    particle.velocity.0 = particle.velocity.0 * drag + dv.0 * dt;
    particle.velocity.1 = particle.velocity.1 * drag + dv.1 * dt;

    particle.center.0 += particle.velocity.0 * dt;
    particle.center.1 += particle.velocity.1 * dt;
}

pub fn update_gravity_particle(particle: &mut Particle) {
    let dt = get_dt(particle);
    let dx = get_distance_to_target(particle);

    let magnitude_dx = magnitude(dx);
    if magnitude_dx < particle.target.size {
        particle.state = ParticleState::ToRemove;
    }

    let dv = (
        dx.0 * particle.target.gravity / magnitude_dx,
        dx.1 * particle.target.gravity / magnitude_dx,
    );
    update_particle_based_on_acceleration(particle, dv, dt, 0.95);
}

pub fn update_linear_particle(particle: &mut Particle) {
    let dt = get_dt(particle);

    particle.center.0 += particle.velocity.0 * dt;
    particle.center.1 += particle.velocity.1 * dt;
    particle.opacity = (particle.opacity - dt).clamp(0_f32, 1_f32);
    //signed distance function without normalization
    //this is the distance between the center and a line though the target,
    //perpendicular to velocity
    let d_x = particle.velocity.1 * (particle.target.center.1 - particle.center.1);
    let d_y = -particle.velocity.0 * (particle.target.center.0 - particle.center.0);
    if d_x - d_y < 0_f32 || particle.opacity == 0_f32 {
        particle.state = ParticleState::ToRemove;
    }
}

pub fn update_repulsed_particle(particle: &mut Particle) {
    let dt = get_dt(particle);
    let dx = get_distance_to_target(particle);
    let mag = magnitude(dx);

    //screen goes from -1 to 1, thus the diagonal is sqrt(2*2 + 2*2)
    if mag > 8_f32.sqrt() {
        particle.state = ParticleState::ToRemove;
    }

    let dv = (
        -dx.0 * particle.target.gravity / mag,
        -dx.1 * particle.target.gravity / mag,
    );

    update_particle_based_on_acceleration(particle, dv, dt, 1.0);
}

fn get_random_point_in_area(point: (f32, f32), area: f32) -> (f32, f32) {
    let mut randomizer = rng();
    let r = randomizer.random_range(0_f32..area / 2_f32);
    let phi = randomizer.random_range(0_f32..2_f32 * PI);

    let x = phi.cos() * r + point.0;
    let y = phi.sin() * r + point.1;
    (x, y)
}

pub fn generate_random_particles_around_point(
    point: (f32, f32),
    area: f32,
    target: Target,
    max_initial_speed: f32,
    opacity: f32,
    color: (f32, f32, f32),
    scale: f32,
    number: usize,
) -> Vec<Particle> {
    let mut result: Vec<Particle> = Vec::with_capacity(number);
    let mut randomizer = rng();

    for _ in 0..number {
        let q = get_random_point_in_area(point, area);
        let v_0: (f32, f32) = (q.0 - point.0, q.1 - point.1);
        let vary = randomizer.random_range(0.5_f32..1.3_f32);
        let v_norm = vary * max_initial_speed / (v_0.0 * v_0.0 + v_0.1 * v_0.1).sqrt();
        let size_vary = randomizer.random_range(0.5_f32..1.5_f32);
        let particle = Particle::new(
            q,
            scale * size_vary,
            color,
            opacity,
            (v_0.0 * v_norm, v_0.1 * v_norm),
            target,
            update_gravity_particle,
        );
        result.push(particle);
    }
    result
}

pub fn generate_random_repulsed_particles_around_point(
    point: (f32, f32),
    area: f32,
    max_initial_speed: f32,
    opacity: f32,
    color: (f32, f32, f32),
    scale: f32,
    number: usize,
) -> Vec<Particle> {
    let mut result: Vec<Particle> = Vec::with_capacity(number);
    let mut randomizer = rng();

    for _ in 0..number {
        let q = get_random_point_in_area(point, area);
        let target = Target {
            center: point,
            gravity: 1_f32,
            size: 1_f32,
        };
        let v_0 = (q.0 - point.0, q.1 - point.1);
        let vary = randomizer.random_range(0.5_f32..1.0_f32);
        let size_vary = randomizer.random_range(0.5_f32..1.5_f32);
        let v_norm = vary * max_initial_speed / (v_0.0 * v_0.0 + v_0.1 * v_0.1).sqrt();
        let particle = Particle::new(
            q,
            scale * size_vary,
            color,
            opacity,
            (v_0.0 * v_norm, v_0.1 * v_norm),
            target,
            update_repulsed_particle,
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
        opacity: f32,
        velocity: (f32, f32),
        target: Target,
        update_function: fn(&mut Particle),
    ) -> Particle {
        Particle {
            color,
            opacity,
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

pub struct ParticleData {
    pub particle_vb: VertexBuffer<ParticleVertex>,
    pub particle_idxb: IndexBuffer<u32>,
    pub particles: Vec<Particle>,
}

pub fn update_particle_state(
    particles: Vec<Particle>,
    display: &DisplayType,
) -> Result<ParticleData, Box<dyn std::error::Error>> {
    let count = particles.len();

    let mut vb: glium::VertexBuffer<ParticleVertex> =
        glium::VertexBuffer::empty_dynamic(display, count * 4)?;
    let mut index_buffer_data: Vec<u32> = Vec::with_capacity(count * 6);
    //we can't map over a Vertex buffer length 0
    if count > 0 {
        create_particle_vertex_buffer(&mut vb, &particles, &mut index_buffer_data);
    }

    let res_vec = particles
        .into_iter()
        .filter(|particle| !matches!(particle.state, ParticleState::ToRemove))
        .collect();

    let idxb: glium::IndexBuffer<u32> = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &index_buffer_data,
    )?;

    Ok(ParticleData {
        particle_vb: vb,
        particle_idxb: idxb,
        particles: res_vec,
    })
}
