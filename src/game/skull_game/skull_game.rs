use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};

use crate::PlatoConfig;
use crate::game::load_shaders;
use crate::game::skull_game::config::SkullSettings;
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

    programs: HashMap<&'static str, glium::Program>,
    texture_arrays: HashMap<&'static str, Texture2dArray>,
    textures: HashMap<&'static str, Texture2d>,

    skull_spawner: SkullSpawner,
    mask: Option<Mat>,
    sound: Option<AudioHandler>,

    settings: SkullSettings,
    difficultiy: DiffcultySelector,
    game_state: Arc<Mutex<GameState>>,
}

impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings: SkullSettings = load_config(config_path)?;
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
                settings: settings.clone(),
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
        scale: f32,
        target_pos: (f32, f32),
        target_scale: f32,
        color: (f32, f32, f32),
    ) -> Vec<Particle> {
        let target = Target {
            center: target_pos,
            gravity: 3.5,
            size: target_scale,
        };
        generate_random_particles_around_point(pos, scale, target, 1.0, color, 0.01, 2000)
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
                        Some(GameEvent::Killed { pos, scale }) => {
                            particles
                                .particles
                                .append(&mut Self::spawn_particles_for_skull(
                                    pos,
                                    scale,
                                    moon_ref.moon.current_position,
                                    moon_ref.moon.scale * 1.2,
                                    (0.0, 1.0, 1.0),
                                ));
                            moon_ref.moon.hit(self.difficultiy.player_damage);
                            sound_ref.play("killed", SoundType::Sfx)?;
                        }
                        Some(GameEvent::Escaped { pos, scale }) => {
                            particles.particles.append(
                                &mut generate_random_repulsed_particles_around_point(
                                    pos,
                                    scale,
                                    1.0_f32,
                                    (1.0, 0.0, 0.0),
                                    0.04,
                                    1200,
                                ),
                            );
                            moon_ref.moon.heal(self.difficultiy.escape_penalty);
                            sound_ref.play("escaped", SoundType::Sfx)?;
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

    fn draw_start(&mut self, frame: &mut glium::Frame) -> Result<(), Box<dyn std::error::Error>> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        match &mut self.live_view_data {
            Some(live) => {
                if let Some(mat) = live.live_view_texture.as_ref() {
                    frame.draw(
                        &live.live_view_vb,
                        &live.live_view_ib,
                        &self.programs["live_program"],
                        &uniform! { live_view_tex: mat},
                        &params,
                    )?
                };
                Ok(())
            }
            None => Err(Self::get_boxed_opencv_error("Live View", 3)),
        }?;

        match &mut self.moon_data {
            Some(moon) => Ok(frame.draw(
                &moon.moon_vb,
                &moon.moon_idxb,
                &self.programs["moon_program"],
                &uniform! {moon_textures: &self.texture_arrays["moon_textures"],time: moon.moon.get_life_fraction()},
                &params,
            )?),
            None => Err(Self::get_boxed_opencv_error("Moon", 3)),
        }?;

        Ok(())
    }

    fn draw_victory(&mut self, frame: &mut glium::Frame) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.victory_data {
            Some(v_data) => Ok(frame.draw(
                &v_data.vertex_buffer,
                &v_data.index_buffer,
                &self.programs["victory_program"],
                &uniform! {tex:&self.textures["victory"]},
                &glium::draw_parameters::DrawParameters::default(),
            )?),
            None => Err(Self::get_boxed_opencv_error("Victory", 3)),
        }
    }

    fn draw_entities(
        &mut self,
        frame: &mut glium::Frame,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };

        match &mut self.live_view_data {
            Some(live) => {
                if let Some(mat) = live.live_view_texture.as_ref() {
                    frame.draw(
                        &live.live_view_vb,
                        &live.live_view_ib,
                        &self.programs["live_program"],
                        &uniform! { live_view_tex: mat},
                        &params,
                    )?
                };
                Ok(())
            }
            None => Err(Self::get_boxed_opencv_error("Live View", 3)),
        }?;

        match &mut self.moon_data {
            Some(moon) => Ok(frame.draw(
                &moon.moon_vb,
                &moon.moon_idxb,
                &self.programs["moon_program"],
                &uniform! {moon_textures: &self.texture_arrays["moon_textures"],time: moon.moon.get_life_fraction()},
                &params,
            )?),
            None => Err(Self::get_boxed_opencv_error("Moon", 3)),
        }?;

        match &self.skull_data {
            Some(skulls) => Ok(frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                &self.programs["skull_program"],
                &uniform! { tex: &self.texture_arrays["skull_textures"], tex_killed: &self.texture_arrays["skull_killed_textures"]},
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
        let moon = Moon::new(20, (0_f32, 0.25_f32), (0_f32, 0.5_f32), 0.2);
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
        load_shader_helper("live_program", &self.settings.live_shader)?;
        load_shader_helper("victory_program", &self.settings.victory_shader)?;

        //load textures
        let mut load_texture_helper =
            |name: &'static str, path| -> Result<(), Box<dyn std::error::Error>> {
                let tex = load_texture(path, self.settings.mask_color, display)?;
                self.texture_arrays.insert(name, tex);
                Ok(())
            };

        load_texture_helper("moon_textures", &self.settings.moon_textures)?;
        load_texture_helper("skull_textures", &self.settings.skull_alive_textures)?;
        load_texture_helper(
            "skull_killed_textures",
            &self.settings.skull_killed_textures,
        )?;

        let mut load_single_texture = |name, path| -> Result<(), Box<dyn std::error::Error>> {
            let tex = load_rgb_image_as_texture(path, display)?;
            self.textures.insert(name, tex);
            Ok(())
        };

        load_single_texture("victory", &self.settings.victory_texture)?;

        //create statefull entitites
        self.skull_data = Some(update_skull_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);

        //get live view data
        self.live_view_data = Some(LiveViewData::generate_vertex_index_buffer(display)?);

        //create particle data
        self.particle_data = Some(update_particle_state(
            Vec::with_capacity(self.settings.max_number),
            display,
        )?);
        self.victory_data = Some(VicotryData::new(display)?);

        //create sound
        self.sound = Some(AudioHandler::new(
            vec![
                (
                    "killed".to_string(),
                    self.settings.skull_killed_sound.clone(),
                ),
                (
                    "escaped".to_string(),
                    self.settings.skull_escaped_sound.clone(),
                ),
            ],
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
                self.draw_start(frame)?;
            }
            GameState::Game => {
                //hit test
                self.hit_test(timestep)?;

                self.moon_data.as_mut().unwrap().moon.life.update();
                //update vertex/index buffer particle_data
                self.update_dynamic_buffers(display)?;

                //draw everything
                self.draw_entities(frame)?;
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
                Key::Character(val) if val.to_lowercase() == self.settings.start_key => {
                    *state = GameState::Game;
                }
                _ => {}
            };
        }

        match event.as_ref() {
            Key::Character("1") => {
                println!("set difficultiy normal");
                self.difficultiy = DiffcultySelector::default();
            }
            Key::Character("2") => {
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
