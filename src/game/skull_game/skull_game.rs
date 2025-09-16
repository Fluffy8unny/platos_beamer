use crate::PlatoConfig;
use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::particle::{
    Particle, ParticleState, ParticleVertex, Target, create_particle_vertex_buffer,
    generate_random_particles_around_point,
};
use crate::game::skull_game::skull::{
    Skull, SkullSpawner, SkullState, SkullVertex, create_skull_vertex_buffer,
};
use crate::game::skull_game::util::load_texture;
use crate::types::game_types::GameTrait;

use ::glium::{IndexBuffer, Surface, VertexBuffer};
use glium::texture::Texture2dArray;
use glium::winit::keyboard::Key;
use opencv::prelude::*;

struct SkullData {
    skull_vb: VertexBuffer<SkullVertex>,
    skull_idxb: IndexBuffer<u16>,
    skulls: Vec<Skull>,
}

struct ParticleData {
    particle_vb: VertexBuffer<ParticleVertex>,
    particle_idxb: IndexBuffer<u16>,
    particles: Vec<Particle>,
}

pub enum GameEvent {
    Killed { pos: (f32, f32), scale: f32 },
    Escaped { pos: (f32, f32), scale: f32 },
}

struct GameState {
    current_score: f32,
}

pub struct SkullGame {
    //2nd rendering path
    skull_data: Option<SkullData>,
    particle_data: Option<ParticleData>,

    skull_program: Option<glium::Program>,
    skull_texture: Option<Texture2dArray>,
    particle_program: Option<glium::Program>,

    skull_spawner: SkullSpawner,
    crystall_position: (f32, f32),
    mask: Option<Mat>,
    settings: SkullSettings,
    game_state: GameState,
}

impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings: SkullSettings = load_config(config_path)?;
        Ok(SkullGame {
            skull_data: None,
            particle_data: None,
            skull_program: None,
            skull_texture: None,
            particle_program: None,
            skull_spawner: SkullSpawner {
                time_since: 0_f32,
                settings: settings.clone(),
            },
            crystall_position: (0_f32, 0_f32),
            mask: None,
            settings,
            game_state: GameState {
                current_score: 0_f32,
            },
        })
    }
}

fn update_skull_state(
    skulls: Vec<Skull>,
    display: &DisplayType,
) -> Result<SkullData, Box<dyn std::error::Error>> {
    let skull_count = skulls.len();

    let mut skull_vb: glium::VertexBuffer<SkullVertex> =
        glium::VertexBuffer::empty_dynamic(display, skull_count * 4)?;
    let mut index_buffer_data: Vec<u16> = Vec::with_capacity(skull_count * 6);
    //we can't map over a Vertex buffer length 0
    if skull_count > 0 {
        create_skull_vertex_buffer(&mut skull_vb, &skulls, &mut index_buffer_data);
    }

    let skull_idxb: glium::IndexBuffer<u16> = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &index_buffer_data,
    )?;

    let res_vec = skulls
        .into_iter()
        .filter(|skull| !matches!(skull.state, SkullState::ToRemove))
        .collect();

    Ok(SkullData {
        skull_vb,
        skull_idxb,
        skulls: res_vec,
    })
}

fn update_particle_state(
    particles: Vec<Particle>,
    display: &DisplayType,
) -> Result<ParticleData, Box<dyn std::error::Error>> {
    let count = particles.len();

    let mut vb: glium::VertexBuffer<ParticleVertex> =
        glium::VertexBuffer::empty_dynamic(display, count * 4)?;
    let mut index_buffer_data: Vec<u16> = Vec::with_capacity(count * 6);
    //we can't map over a Vertex buffer length 0
    if count > 0 {
        create_particle_vertex_buffer(&mut vb, &particles, &mut index_buffer_data);
    }

    let res_vec = particles
        .into_iter()
        .filter(|particle| !matches!(particle.state, ParticleState::ToRemove))
        .collect();

    let idxb: glium::IndexBuffer<u16> = glium::IndexBuffer::new(
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

fn spawn_particles_for_skull(
    pos: (f32, f32),
    scale: f32,
    target_pos: (f32, f32),
    color: (f32, f32, f32),
) -> Vec<Particle> {
    let target = Target {
        center: target_pos,
        gravity: 0.5,
        size: 0.1,
    };
    generate_random_particles_around_point(pos, scale, target, 0.6, color, 0.01, 250)
}

impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        _config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let particle_program = load_shaders(&self.settings.particle_shader, display)?;
        self.particle_program = Some(particle_program);

        let skull_program = load_shaders(&self.settings.skull_shader, display)?;
        self.skull_texture = Some(load_texture(
            &self.settings.skull_alive_textures,
            self.settings.mask_color,
            display,
        )?);
        self.skull_program = Some(skull_program);
        self.skull_data = Some(update_skull_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);

        self.particle_data = Some(update_particle_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);
        Ok(())
    }

    fn update(
        &mut self,
        _image: &Mat,
        mask: &Mat,
        _display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mask = Some(mask.clone());
        Ok(())
    }
    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        display: &DisplayType,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //update skulls
        match (&mut self.skull_data, &mut self.particle_data) {
            (Some(data), Some(particles)) => {
                for skull in data.skulls.iter_mut() {
                    match skull.update(&self.mask, timestep)? {
                        Some(GameEvent::Killed { pos, scale }) => {
                            particles.particles.append(&mut spawn_particles_for_skull(
                                pos,
                                scale,
                                self.crystall_position,
                                (1.0, 1.0, 0.0),
                            ));
                        }
                        Some(GameEvent::Escaped { pos, scale }) => {
                            particles.particles.append(&mut spawn_particles_for_skull(
                                pos,
                                scale,
                                self.crystall_position,
                                (0.0, 1.0, 1.0),
                            ));
                        }
                        None => {}
                    }
                }
                for particle in particles.particles.iter_mut() {
                    particle.update()
                }

                self.skull_spawner.maybe_spawn(&mut data.skulls, &timestep);
                Ok(())
            }
            (_, None) => Err(Box::new(opencv::Error {
                message: "Particle data was not initialized".to_string(),
                code: 3,
            })),
            (None, _) => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }?;

        //todo fix this mess....
        //update buffer data
        self.skull_data = Some(update_skull_state(
            self.skull_data.as_ref().unwrap().skulls.clone(),
            display,
        )?);

        self.particle_data = Some(update_particle_state(
            self.particle_data.as_ref().unwrap().particles.clone(),
            display,
        )?);

        //draw skulls
        let skull_program = self
            .skull_program
            .as_ref()
            .ok_or(Box::new(opencv::Error::new(4, "Skull program not loaded.")))?;
        match &self.skull_data {
            Some(skulls) => Ok(frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                skull_program,
                &glium::uniforms::EmptyUniforms,
                &glium::DrawParameters::default(),
            )?),
            None => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }?;
        let particle_program =
            self.particle_program
                .as_ref()
                .ok_or(Box::new(opencv::Error::new(
                    4,
                    "Particle program not loaded.",
                )))?;
        match &self.particle_data {
            Some(particles) => Ok(frame.draw(
                &particles.particle_vb,
                &particles.particle_idxb,
                particle_program,
                &glium::uniforms::EmptyUniforms,
                &glium::DrawParameters::default(),
            )?),
            None => Err(Box::new(opencv::Error {
                message: "Particle data was not initialized".to_string(),
                code: 3,
            })),
        }
    }
    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
