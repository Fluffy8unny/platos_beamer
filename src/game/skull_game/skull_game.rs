use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};

use crate::PlatoConfig;
use crate::game::load_shaders;
use crate::game::skull_game::config::{GameSettings, ParticleSetting};
use crate::game::skull_game::live_view::LiveViewData;
use crate::game::skull_game::moon::{Moon, MoonData, create_moon_vertex_buffer, update_moon_data};
use crate::game::skull_game::particle::{
    Particle, ParticleData, Target, generate_random_particles_around_point,
    generate_random_repulsed_particles_around_point, update_particle_state,
};
use crate::game::skull_game::position_visualization::spawn_based_on_mask;
use crate::game::skull_game::skull::{GameEvent, SkullData, SkullSpawner, update_skull_state};
use crate::game::skull_game::util::load_texture;
use crate::game::skull_game::victory::VicotryData;
use crate::game::sound::{AudioHandler, SoundType};
use crate::game::util::load_rgb_image_as_texture;
use crate::types::game_types::GameTrait;

use opencv::prelude::*;

use ::glium::{Surface, uniform};
use glium::texture::{Texture2d, Texture2dArray};
use glium::winit::keyboard::Key;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
enum GameState {
    PreGame,
    Game,
    PostGame,
}

struct DiffcultySelector {
    player_damage: u32,
    escape_penalty: u32,
}

impl DiffcultySelector {
    pub fn default() -> DiffcultySelector {
        DiffcultySelector {
            player_damage: 1,
            escape_penalty: 1,
        }
    }
}

pub struct SkullGame {
    skull_data: Option<SkullData>,
    particle_data: Option<ParticleData>,
    live_view_data: Option<LiveViewData>,
    moon_data: Option<MoonData>,
    victory_data: Option<VicotryData>,

    programs: HashMap<String, glium::Program>,
    texture_arrays: HashMap<String, Texture2dArray>,
    textures: HashMap<String, Texture2d>,

    skull_spawner: SkullSpawner,
    mask: Option<Mat>,
    sound: Option<AudioHandler>,

    settings: GameSettings,
    difficultiy: DiffcultySelector,
    game_state: Arc<Mutex<GameState>>,
}

impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings: GameSettings = load_config(config_path)?;
        Ok(SkullGame {
            skull_data: None,
            particle_data: None,
            live_view_data: None,
            moon_data: None,
            victory_data: None,
            programs: HashMap::new(),
            texture_arrays: HashMap::new(),
            textures: HashMap::new(),
            skull_spawner: SkullSpawner {
                time_since: 0_f32,
                settings: settings.skull_settings.clone(),
            },
            mask: None,
            settings,
            sound: None,
            difficultiy: DiffcultySelector::default(),
            game_state: Arc::new(Mutex::new(GameState::PreGame)),
        })
    }

    fn spawn_particles_for_skull(
        pos: (f32, f32),
        skull_scale: f32,
        target_pos: (f32, f32),
        target_scale: f32,
        settings: &ParticleSetting,
    ) -> Vec<Particle> {
        let target = Target {
            center: target_pos,
            gravity: 3.5,
            size: target_scale,
        };
        generate_random_particles_around_point(
            pos,
            skull_scale,
            target,
            settings.initial_velocity,
            settings.color,
            settings.scale,
            settings.number,
        )
    }

    fn get_boxed_opencv_error(name: &str, code: i32) -> Box<opencv::Error> {
        Box::new(opencv::Error {
            message: format!("{} data was not initialized", name).to_string(),
            code,
        })
    }

    fn hit_test(&mut self, timestep: &TimeStep) -> Result<(), Box<dyn std::error::Error>> {
        let moon_ref: &mut MoonData = self.moon_data.as_mut().ok_or("moon not defined")?;
        let sound_ref = self.sound.as_ref().ok_or("sound not initialized")?;

        //hit test
        match (&mut self.skull_data, &mut self.particle_data) {
            (Some(data), Some(particles)) => {
                for skull in data.skulls.iter_mut() {
                    match skull.update(&self.mask, timestep)? {
                        Some(GameEvent::Killed { pos, skull_scale }) => {
                            particles
                                .particles
                                .append(&mut Self::spawn_particles_for_skull(
                                    pos,
                                    skull_scale,
                                    moon_ref.moon.current_position,
                                    moon_ref.moon.scale * 1.2,
                                    &self.settings.particle_settings.killed,
                                ));
                            moon_ref.moon.hit(self.difficultiy.player_damage);
                            sound_ref.play("skull_kill_sound", SoundType::Sfx)?;
                        }
                        Some(GameEvent::Escaped { pos, scale }) => {
                            particles.particles.append(
                                &mut generate_random_repulsed_particles_around_point(
                                    pos,
                                    scale,
                                    self.settings.particle_settings.escaped.initial_velocity,
                                    self.settings.particle_settings.escaped.color,
                                    self.settings.particle_settings.escaped.scale,
                                    self.settings.particle_settings.escaped.number,
                                ),
                            );
                            moon_ref.moon.heal(self.difficultiy.escape_penalty);
                            sound_ref.play("skull_escaped_sound", SoundType::Sfx)?;
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
            (_, None) => Err(Self::get_boxed_opencv_error("Particle", 3)),
            (None, _) => Err(Self::get_boxed_opencv_error("Skull", 3)),
        }?;
        Ok(())
    }

    fn draw_live(
        &mut self,
        frame: &mut glium::Frame,
        params: &glium::DrawParameters,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.live_view_data {
            Some(live) => {
                if let Some(mat) = live.live_view_texture.lock().unwrap().as_ref() {
                    frame.draw(
                        &live.live_view_vb,
                        &live.live_view_ib,
                        &self.programs["live_program"],
                    &uniform! { live_tex: mat, clouds: &self.textures["clouds"], time: timestep.runtime*0.001 },
                        &params,
                    )?
                };
                Ok(())
            }
            None => Err(Self::get_boxed_opencv_error("Live View", 3)),
        }
    }

    fn draw_moon(
        &mut self,
        frame: &mut glium::Frame,
        params: &glium::DrawParameters,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.moon_data {
            Some(moon) => Ok(frame.draw(
                &moon.moon_vb,
                &moon.moon_idxb,
                &self.programs["moon_program"],
                &uniform! {moon_textures: &self.texture_arrays["moon_textures"],time: moon.moon.get_life_fraction()},
                &params,
            )?),
            None => Err(Self::get_boxed_opencv_error("Moon", 3)),
        }
    }

    fn draw_start(
        &mut self,
        frame: &mut glium::Frame,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        self.draw_live(frame, &params, timestep)?;
        self.draw_moon(frame, &params)
    }

    fn draw_victory(&mut self, frame: &mut glium::Frame) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.victory_data {
            Some(v_data) => Ok(frame.draw(
                &v_data.vertex_buffer,
                &v_data.index_buffer,
                &self.programs["victory_program"],
                &uniform! {tex:&self.textures["victory_texture"]},
                &glium::draw_parameters::DrawParameters::default(),
            )?),
            None => Err(Self::get_boxed_opencv_error("Victory", 3)),
        }
    }

    fn draw_entities(
        &mut self,
        frame: &mut glium::Frame,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        self.draw_live(frame, &params, timestep)?;
        self.draw_moon(frame, &params)?;

        match &self.skull_data {
            Some(skulls) => Ok(frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                &self.programs["skull_program"],
                &uniform! { tex: &self.texture_arrays["skull_alive_textures"], tex_killed: &self.texture_arrays["skull_killed_textures"]},
                &params
            )?),
            None => Err(Self::get_boxed_opencv_error("Skull",3)),
        }?;

        match &self.particle_data {
            Some(particles) => Ok(frame.draw(
                &particles.particle_vb,
                &particles.particle_idxb,
                &self.programs["particle_program"],
                &glium::uniforms::EmptyUniforms,
                &params,
            )?),
            None => Err(Self::get_boxed_opencv_error("Particle", 3)),
        }
    }

    fn update_dynamic_buffers(
        &mut self,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.skull_data = Some(update_skull_state(
            self.skull_data.as_ref().unwrap().skulls.clone(),
            display,
        )?);

        self.particle_data = Some(update_particle_state(
            self.particle_data.as_ref().unwrap().particles.clone(),
            display,
        )?);
        self.moon_data.as_mut().unwrap().moon.update_position();
        self.moon_data = Some(update_moon_data(self.moon_data.as_ref().unwrap(), display)?);
        Ok(())
    }
}

impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let moon = Moon::new(self.settings.moon_settings.clone());
        let (moon_vb, moon_idxb) = create_moon_vertex_buffer(&moon, display)?;
        self.moon_data = Some(MoonData {
            moon_vb,
            moon_idxb,
            moon,
        });

        //load shaders
        let mut load_shader_helper =
            |name: String, path: String| -> Result<(), Box<dyn std::error::Error>> {
                let program = load_shaders(&path, display)?;
                self.programs.insert(name, program);
                Ok(())
            };
        for (k, v) in self.settings.shader_settings.clone() {
            load_shader_helper(k, v)?;
        }

        //load textures
        let mut load_texture_helper =
            |name: String, path: Vec<String>| -> Result<(), Box<dyn std::error::Error>> {
                let tex = load_texture(&path, self.settings.texture_settings.mask_color, display)?;
                self.texture_arrays.insert(name, tex);
                Ok(())
            };
        for (k, v) in self.settings.texture_settings.texture_arrays.clone() {
            load_texture_helper(k, v)?;
        }

        let mut load_single_texture =
            |name, path: String| -> Result<(), Box<dyn std::error::Error>> {
                let tex = load_rgb_image_as_texture(&path, display)?;
                self.textures.insert(name, tex);
                Ok(())
            };
        for (k, v) in self.settings.texture_settings.textures.clone() {
            load_single_texture(k, v)?;
        }

        //create statefull entitites
        self.skull_data = Some(update_skull_state(
            Vec::with_capacity(self.settings.skull_settings.max_number),
            display,
        )?);

        //get live view data
        self.live_view_data = Some(LiveViewData::generate_vertex_index_buffer(display)?);

        //create particle data
        self.particle_data = Some(update_particle_state(
            Vec::with_capacity(self.settings.skull_settings.max_number),
            display,
        )?);
        self.victory_data = Some(VicotryData::new(display)?);

        //create sound
        self.sound = Some(AudioHandler::new(
            self.settings.sound_settings.clone(),
            config.sound_config,
        )?);
        Ok(())
    }

    fn update(
        &mut self,
        image: &Mat,
        mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mask = Some(mask.clone());
        if let Some(lv_ref) = self.live_view_data.as_mut() {
            lv_ref.set_live_view_texture(display, image)?
        };
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
            if let Ok(mut motion_particles) = spawn_based_on_mask(mask, 800) {
                if let Some(particle_data) = &mut self.particle_data {
                    particle_data.particles.append(&mut motion_particles);
                }
            }
        }

        let state_mut = self.game_state.clone();
        let mut state = state_mut.lock().unwrap();
        match *state {
            GameState::PreGame => {
                self.draw_start(frame, timestep)?;
            }

            GameState::Game => {
                //hit test
                self.hit_test(timestep)?;

                self.moon_data.as_mut().unwrap().moon.life.update();
                //update vertex/index buffer particle_data
                self.update_dynamic_buffers(display)?;

                //draw everything
                self.draw_entities(frame, timestep)?;
                //check for win condition
                if let Some(moon_d) = self.moon_data.as_ref() {
                    if moon_d.moon.life.current_value == 0_f32 {
                        *state = GameState::PostGame;
                    }
                }
            }

            GameState::PostGame => {
                self.draw_victory(frame)?;
            }
        };
        Ok(())
    }

    fn key_event(&mut self, event: &Key) {
        let mut state = self.game_state.lock().unwrap();
        if let GameState::PreGame = *state {
            match event.as_ref() {
                Key::Character(val)
                    if val.to_lowercase() == self.settings.key_settings.start_key =>
                {
                    *state = GameState::Game;
                }
                _ => {}
            };
        }

        match event.as_ref() {
            Key::Character(val) if val == self.settings.key_settings.normal_mode_key => {
                println!("set difficultiy normal");
                self.difficultiy = DiffcultySelector::default();
            }
            Key::Character(val) if val == self.settings.key_settings.easy_mode_key => {
                println!("set difficultiy easy");
                self.difficultiy = DiffcultySelector {
                    player_damage: 5,
                    escape_penalty: 0,
                };
            }
            _ => {}
        };
    }

    fn reset(&mut self) {
        if let Some(moon_d) = self.moon_data.as_mut() {
            moon_d.moon.life.reset(moon_d.moon.max_life as f32);
            moon_d.moon.current_position = moon_d.moon.position;
        }

        if let Some(part_d) = self.particle_data.as_mut() {
            part_d.particles.clear();
        }

        if let Some(skull_d) = self.skull_data.as_mut() {
            skull_d.skulls.clear();
        }

        *self.game_state.lock().unwrap() = GameState::PreGame;
    }
}
