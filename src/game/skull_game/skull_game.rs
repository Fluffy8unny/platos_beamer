use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};

use crate::PlatoConfig;
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
use crate::game::skull_game::moon::{Moon, MoonData, create_moon_vertex_buffer};
use crate::game::skull_game::particle::{
    Particle, ParticleData, Target, generate_random_particles_around_point,
    generate_random_repulsed_particles_around_point, update_particle_state,
};
use crate::game::skull_game::position_visualization::spawn_based_on_mask;
use crate::game::skull_game::skull::{GameEvent, SkullData, SkullSpawner, update_skull_state};
use crate::game::skull_game::util::load_texture;
use crate::game::sound::{AudioHandler, SoundType};
use crate::types::game_types::GameTrait;

use opencv::prelude::*;

use ::glium::{Surface, uniform};
use glium::texture::Texture2dArray;
use glium::winit::keyboard::Key;

use std::collections::HashMap;
pub struct SkullGame {
    skull_data: Option<SkullData>,
    particle_data: Option<ParticleData>,
    moon_data: Option<MoonData>,

    programs: HashMap<&'static str, glium::Program>,
    textures: HashMap<&'static str, Texture2dArray>,

    skull_spawner: SkullSpawner,
    mask: Option<Mat>,
    sound: Option<AudioHandler>,

    settings: SkullSettings,
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

    fn spawn_particles_for_skull(
        pos: (f32, f32),
        scale: f32,
        target_pos: (f32, f32),
        color: (f32, f32, f32),
    ) -> Vec<Particle> {
        let target = Target {
            center: target_pos,
            gravity: 3.5,
            size: 0.2,
        };
        generate_random_particles_around_point(pos, scale, target, 1.0, color, 0.01, 2000)
    }

    fn hit_test(&mut self, timestep: &TimeStep) -> Result<(), Box<dyn std::error::Error>> {
        //get reference to the moon
        let moon_ref: &mut MoonData = self.moon_data.as_mut().ok_or("moon not defined")?;
        //reference to the sound controller
        let sound_ref = self.sound.as_ref().ok_or("sound not initialized")?;

        //hit test
        match (&mut self.skull_data, &mut self.particle_data) {
            (Some(data), Some(particles)) => {
                for skull in data.skulls.iter_mut() {
                    match skull.update(&self.mask, timestep)? {
                        Some(GameEvent::Killed { pos, scale }) => {
                            particles
                                .particles
                                .append(&mut Self::spawn_particles_for_skull(
                                    pos,
                                    scale,
                                    moon_ref.moon.position,
                                    (0.0, 1.0, 1.0),
                                ));
                            moon_ref.moon.hit(1);
                            sound_ref.play("killed", SoundType::Sfx)?;
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
                            sound_ref.play("killed", SoundType::Sfx)?;
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
        Ok(())
    }

    fn draw_entities(&self, frame: &mut glium::Frame) -> Result<(), Box<dyn std::error::Error>> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };

        match &self.moon_data{
            Some( moon) => Ok( frame.draw( 
                    &moon.moon_vb,
                    &moon.moon_idxb, 
                    &self.programs["moon_program"], 
                    &uniform! {moon_textures: &self.textures["moon_textures"], moon_maks:&self.textures["moon_masks"]},
                    &params)?
            ),
            None => Err(Box::new(opencv::Error {
                message: "Moon data was not initialized".to_string(),
                code: 3,
            })),
        }?;

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
}

impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let moon = Moon::new(100, (0_f32, 0_f32), 0.5);
        let (moon_vb, moon_idxb) = create_moon_vertex_buffer(&moon, display)?;
        self.moon_data = Some(MoonData {
            moon_vb,
            moon_idxb,
            moon,
        });

        //load shaders
        let mut load_shader_helper =
            |name: &'static str, path: &str| -> Result<(), Box<dyn std::error::Error>> {
                let program = load_shaders(path, display)?;
                self.programs.insert(name, program);
                Ok(())
            };

        load_shader_helper("skull_program", &self.settings.skull_shader)?;
        load_shader_helper("particle_program", &self.settings.particle_shader)?;
        load_shader_helper("moon_program", &self.settings.moon_shader)?;

        //load textures
        let mut load_texture_helper =
            |name: &'static str, path| -> Result<(), Box<dyn std::error::Error>> {
                let tex = load_texture(path, self.settings.mask_color, display)?;
                self.textures.insert(name, tex);
                Ok(())
            };
        load_texture_helper("moon_textures", &self.settings.moon_textures)?;
        load_texture_helper("moon_masks", &self.settings.moon_masks)?;
        load_texture_helper("skull_textures", &self.settings.skull_alive_textures)?;
        load_texture_helper(
            "skull_killed_textures",
            &self.settings.skull_killed_textures,
        )?;

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

        //hit test
        self.hit_test(timestep)?;

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

        //draw everything
        self.draw_entities(frame)?;
        Ok(())
    }
    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
