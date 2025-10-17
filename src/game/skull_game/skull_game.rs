use crate::PlatoConfig;
use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::moon::{Moon, MoonVertex, create_moon_vertex_buffer};
use crate::game::skull_game::particle::{
    Particle, ParticleState, ParticleVertex, Target, create_particle_vertex_buffer,
    generate_random_particles_around_point, generate_random_repulsed_particles_around_point,
};
use crate::game::skull_game::position_visualization::spawn_based_on_mask;
use crate::game::skull_game::skull::{
    Skull, SkullSpawner, SkullState, SkullVertex, create_skull_vertex_buffer,
};
use crate::game::skull_game::util::load_texture;
use crate::game::sound::{AudioHandler, SoundType};
use crate::types::game_types::GameTrait;

use ::glium::{IndexBuffer, Surface, VertexBuffer, uniform};
use glium::texture::Texture2dArray;
use glium::winit::keyboard::Key;
use opencv::prelude::*;
use std::collections::HashMap;

struct SkullData {
    skull_vb: VertexBuffer<SkullVertex>,
    skull_idxb: IndexBuffer<u32>,
    skulls: Vec<Skull>,
}

struct MoonData {
    moon_vb: VertexBuffer<MoonVertex>,
    moon_idxb: IndexBuffer<u32>,
    moon: Moon,
}

struct ParticleData {
    particle_vb: VertexBuffer<ParticleVertex>,
    particle_idxb: IndexBuffer<u32>,
    particles: Vec<Particle>,
}

pub enum GameEvent {
    Killed { pos: (f32, f32), scale: f32 },
    Escaped { pos: (f32, f32), scale: f32 },
}

pub struct SkullGame {
    skull_data: Option<SkullData>,
    particle_data: Option<ParticleData>,

    moon_data: Option<MoonData>,
    programs: HashMap<&'static str, glium::Program>,
    textures: HashMap<&'static str, Texture2dArray>,

    skull_spawner: SkullSpawner,
    mask: Option<Mat>,
    settings: SkullSettings,
    sound: Option<AudioHandler>,
}

impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings: SkullSettings = load_config(config_path)?;
        Ok(SkullGame {
            skull_data: None,
            particle_data: None,
            moon_data: None,
            programs: HashMap::new(),
            textures: HashMap::new(),
            skull_spawner: SkullSpawner {
                time_since: 0_f32,
                settings: settings.clone(),
            },
            mask: None,
            settings,
            sound: None,
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
    let mut index_buffer_data: Vec<u32> = Vec::with_capacity(skull_count * 6);
    //we can't map over a Vertex buffer length 0
    if skull_count > 0 {
        create_skull_vertex_buffer(&mut skull_vb, &skulls, &mut index_buffer_data);
    }

    let skull_idxb: glium::IndexBuffer<u32> = glium::IndexBuffer::new(
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

fn spawn_particles_for_skull(
    pos: (f32, f32),
    scale: f32,
    target_pos: (f32, f32),
    color: (f32, f32, f32),
) -> Vec<Particle> {
    let target = Target {
        center: target_pos,
        gravity: 3.5,
        size: 0.1,
    };
    generate_random_particles_around_point(pos, scale, target, 1.0, color, 0.01, 2000)
}

impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let moon = Moon::new(100, (0_f32, 0_f32), 0.2);
        let (moon_vb, moon_idxb) = create_moon_vertex_buffer(&moon, display)?;
        self.moon_data = Some(MoonData {
            moon_vb,
            moon_idxb,
            moon,
        });

        //load shaders
        let skull_program = load_shaders(&self.settings.skull_shader, display)?;
        let particle_program = load_shaders(&self.settings.particle_shader, display)?;
        self.programs.insert("skull_program", skull_program);
        self.programs.insert("particle_program", particle_program);

        //load textures
        let load_texture_helper = |path| load_texture(path, self.settings.mask_color, display);
        let skull_texture = load_texture_helper(&self.settings.skull_alive_textures)?;
        let skull_killed_texture = load_texture_helper(&self.settings.skull_killed_textures)?;
        self.textures.insert("skull_textures", skull_texture);
        self.textures
            .insert("skull_killed_textures", skull_killed_texture);

        //create statefull entitites
        self.skull_data = Some(update_skull_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);

        self.particle_data = Some(update_particle_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);
        //create sound
        self.sound = Some(AudioHandler::new(
            vec![(
                "killed".to_string(),
                self.settings.skull_killed_sound.clone(),
            )],
            config.sound_config,
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
        //particles for pos visialization
        if let Some(mask) = &self.mask {
            if let Ok(mut motion_particles) = spawn_based_on_mask(mask, 200) {
                if let Some(particle_data) = &mut self.particle_data {
                    particle_data.particles.append(&mut motion_particles);
                }
            }
        }
        //get reference to the moon
        let moon_position = self
            .moon_data
            .as_ref()
            .ok_or("moon not defined")?
            .moon
            .position;
        //draw moon

        //update skulls
        match (&mut self.skull_data, &mut self.particle_data) {
            (Some(data), Some(particles)) => {
                for skull in data.skulls.iter_mut() {
                    match skull.update(&self.mask, timestep)? {
                        Some(GameEvent::Killed { pos, scale }) => {
                            particles.particles.append(&mut spawn_particles_for_skull(
                                pos,
                                scale,
                                moon_position,
                                (0.0, 1.0, 1.0),
                            ));
                            self.sound
                                .as_ref()
                                .ok_or("sound not initialized")?
                                .play("killed", SoundType::Sfx)?;
                        }
                        Some(GameEvent::Escaped { pos, scale }) => {
                            particles.particles.append(
                                &mut generate_random_repulsed_particles_around_point(
                                    pos,
                                    scale,
                                    1.0_f32,
                                    (1.0, 0.0, 0.0),
                                    0.02,
                                    800,
                                ),
                            );
                            self.sound
                                .as_ref()
                                .ok_or("sound not initialized")?
                                .play("killed", SoundType::Sfx)?;
                        }
                        None => {}
                    }
                }
                for particle in particles.particles.iter_mut() {
                    particle.update()
                }

                self.skull_spawner.maybe_spawn(&mut data.skulls, timestep);
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
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        match &self.skull_data {
            Some(skulls) => Ok(frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                &self.programs["skull_program"],
                &uniform! { tex: &self.textures["skull_textures"], tex_killed: &self.textures["skull_killed_textures"]},
                &params
            )?),
            None => Err(Box::new(opencv::Error {
                message: "Skull data was not initialized".to_string(),
                code: 3,
            })),
        }?;
        match &self.particle_data {
            Some(particles) => Ok(frame.draw(
                &particles.particle_vb,
                &particles.particle_idxb,
                &self.programs["particle_program"],
                &glium::uniforms::EmptyUniforms,
                &params,
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
