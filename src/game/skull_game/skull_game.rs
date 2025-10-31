use crate::PlatoConfig;
use crate::config::load_config;
use crate::display::{display_window::DisplayType, timestep::TimeStep};
use crate::game::load_shaders;
use crate::game::skull_game::config::{DifficultySelector, GameSettings, valdiate_config};
use crate::game::skull_game::live_view::LiveViewData;
use crate::game::skull_game::moon::{MoonData, create_moon_data, update_moon_data};
use crate::game::skull_game::particle::{
    ParticleData, generate_random_repulsed_particles_around_point, spawn_particles_for_skull,
    update_particle_state,
};
use crate::game::skull_game::position_visualization::spawn_based_on_mask;
use crate::game::skull_game::skull::{
    self, GameEvent, SkullData, SkullSpawner, update_skull_state,
};
use crate::game::skull_game::util::{
    get_boxed_opencv_error, get_draw_params, get_random_sound_name, load_texture,
};
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
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
struct RoundCounter {
    round: u32,
    max_round: u32,
    time_info: Instant,
}

impl RoundCounter {
    pub fn new(round: u32, settings: &GameSettings) -> RoundCounter {
        RoundCounter {
            round,
            max_round: settings.number_of_rounds,
            time_info: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GameState {
    PreGame,
    Game(RoundCounter),
    Intermission(RoundCounter),
    PostGame(RoundCounter),
}

pub struct SkullGame {
    skull_spawner: SkullSpawner,
    skull_data: Option<SkullData>,
    particle_data: Option<ParticleData>,
    live_view_data: Option<LiveViewData>,
    moon_data: Option<MoonData>,
    victory_data: Option<VicotryData>,
    sound: Option<AudioHandler>,

    programs: HashMap<String, glium::Program>,
    texture_arrays: HashMap<String, Texture2dArray>,
    textures: HashMap<String, Texture2d>,

    mask: Option<Mat>,

    settings: GameSettings,
    difficultiy: DifficultySelector,
    game_state: Arc<Mutex<GameState>>,
}

impl SkullGame {
    pub fn new(config_path: &str) -> Result<SkullGame, Box<dyn std::error::Error>> {
        let settings: GameSettings = load_config(config_path)?;
        valdiate_config(&settings)?;
        let difficulty = settings.difficultiy_settings.normal;

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
            difficultiy: difficulty,
            game_state: Arc::new(Mutex::new(GameState::PreGame)),
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
                            particles.particles.append(&mut spawn_particles_for_skull(
                                pos,
                                skull_scale,
                                moon_ref.moon.current_position,
                                (
                                    1.2_f32 * moon_ref.moon.scale.0,
                                    1.2_f32 * moon_ref.moon.scale.1,
                                ),
                                &self.settings.particle_settings.killed,
                            ));
                            moon_ref.moon.hit(self.difficultiy.player_damage);

                            sound_ref.play(
                                &get_random_sound_name(
                                    "skull_kill_sound",
                                    self.settings.number_of_kill_sounds,
                                ),
                                SoundType::Sfx,
                            )?;
                        }
                        Some(GameEvent::Escaped { pos, scale }) => {
                            particles.particles.append(
                                &mut generate_random_repulsed_particles_around_point(
                                    pos,
                                    scale,
                                    &self.settings.particle_settings.escaped,
                                ),
                            );
                            moon_ref.moon.heal(self.difficultiy.escape_penalty);
                            sound_ref.play(
                                &get_random_sound_name(
                                    "skull_escaped_sound",
                                    self.settings.number_of_escape_sounds,
                                ),
                                SoundType::Sfx,
                            )?;
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
            (_, None) => Err(get_boxed_opencv_error("Particle", 3)),
            (None, _) => Err(get_boxed_opencv_error("Skull", 3)),
        }?;
        Ok(())
    }

    fn draw_live(
        &mut self,
        frame: &mut glium::Frame,
        params: &glium::DrawParameters,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match (&mut self.live_view_data, &self.moon_data) {
            (Some(live), Some(moon)) => {
                if let Some(mat) = live.live_view_texture.lock().unwrap().as_ref() {
                    frame.draw(
                        &live.live_view_vb,
                        &live.live_view_ib,
                        &self.programs["live_program"],
                        &uniform! { live_tex: mat, moon_texture: &self.textures["moon_texture"],
                        clouds: &self.textures["clouds"], clouds2: &self.textures["clouds2"],
                        sky: &self.textures["sky"], moon_pos_u: moon.moon.get_position(),
                        moon_scale:moon.moon.scale, time: timestep.runtime*0.001 },
                        params,
                    )?
                };
                Ok(())
            }
            _ => Err(get_boxed_opencv_error("Live View or Moon", 3)),
        }
    }

    fn draw_moon(
        &mut self,
        frame: &mut glium::Frame,
        params: &glium::DrawParameters,
        timestep: &TimeStep,
        color_selector: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.moon_data {
            Some(moon) => {
                frame.draw(
                    &moon.corona_vb,
                    &moon.corona_idxb,
                    &self.programs["corona_program"],
                    &uniform! {time: timestep.runtime / 1000_f32,
                    life: moon.moon.get_life_fraction(),
                    corona_color: moon.moon.corona_color[color_selector]},
                    params,
                )?;

                frame.draw(
                    &moon.moon_vb,
                    &moon.moon_idxb,
                    &self.programs["moon_program"],
                    &uniform! {moon_texture: &self.textures["moon_texture"],
                    moon_mask: &self.textures["moon"],
                    color_overlay: moon.moon.color_overlay[color_selector],
                    time: moon.moon.get_life_fraction()},
                    params,
                )?;
                Ok(())
            }
            None => Err(get_boxed_opencv_error("Moon", 3)),
        }
    }

    fn draw_scenary(
        &mut self,
        frame: &mut glium::Frame,
        timestep: &TimeStep,
        round_counter: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let params = get_draw_params();
        self.draw_live(frame, &params, timestep)?;
        self.draw_moon(frame, &params, timestep, round_counter)
    }

    fn draw_victory(&mut self, frame: &mut glium::Frame) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.victory_data {
            Some(v_data) => Ok(frame.draw(
                &v_data.vertex_buffer,
                &v_data.index_buffer,
                &self.programs["victory_program"],
                &uniform! {tex:&self.textures["victory_texture"]},
                &get_draw_params(),
            )?),
            None => Err(get_boxed_opencv_error("Victory", 3)),
        }
    }

    fn draw_particles(
        &mut self,
        frame: &mut glium::Frame,
        params: &glium::DrawParameters,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &self.particle_data {
            Some(particles) => Ok(frame.draw(
                &particles.particle_vb,
                &particles.particle_idxb,
                &self.programs["particle_program"],
                &glium::uniforms::EmptyUniforms,
                &params,
            )?),
            None => Err(get_boxed_opencv_error("Particle", 3)),
        }
    }

    fn draw_skulls(
        &mut self,
        frame: &mut glium::Frame,
        params: &glium::DrawParameters,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &self.skull_data {
            Some(skulls) => Ok(frame.draw(
                &skulls.skull_vb,
                &skulls.skull_idxb,
                &self.programs["skull_program"],
                &uniform! { tex: &self.texture_arrays["skull_alive_textures"],
                tex_killed: &self.texture_arrays["skull_killed_textures"]},
                &params,
            )?),
            None => Err(get_boxed_opencv_error("Skull", 3)),
        }
    }

    fn update_dynamic_buffers(
        &mut self,
        display: &DisplayType,
        time_step: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.skull_data = Some(update_skull_state(
            self.skull_data.as_ref().unwrap().skulls.clone(),
            display,
        )?);

        self.particle_data = Some(update_particle_state(
            self.particle_data.as_ref().unwrap().particles.clone(),
            display,
        )?);

        self.moon_data = Some(update_moon_data(
            self.moon_data.as_mut().unwrap(),
            display,
            time_step,
        )?);
        Ok(())
    }

    fn handle_mask(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(mask), Some(particle_data)) = (&self.mask, &mut self.particle_data) {
            if let Ok(mut motion_particles) =
                spawn_based_on_mask(mask, self.settings.particle_settings.visualization.number)
            {
                particle_data.particles.append(&mut motion_particles);
            }
        }
        Ok(())
    }

    fn kill_all_skulls(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(moon_d), Some(skull_d), Some(particle_d)) = (
            self.moon_data.as_mut(),
            self.skull_data.as_mut(),
            self.particle_data.as_mut(),
        ) {
            for skull in skull_d.skulls.iter_mut() {
                particle_d.particles.append(&mut spawn_particles_for_skull(
                    skull.center,
                    skull.scale,
                    moon_d.moon.current_position,
                    (1.2_f32 * moon_d.moon.scale.0, 1.2_f32 * moon_d.moon.scale.1),
                    &self.settings.particle_settings.killed,
                ));
                skull.state = skull::SkullState::Killed;
            }
            skull_d.skulls.clear();
        };
        Ok(())
    }
}

impl GameTrait for SkullGame {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        //create moon data
        self.moon_data = Some(create_moon_data(display, &self.settings.moon_settings)?);
        //create skull data
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
        time_step: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state_mut = self.game_state.clone();
        let mut state = state_mut.lock().unwrap();
        let sound_ref = self.sound.as_ref().ok_or("sound not inited")?;
        let params = get_draw_params();

        match *state {
            GameState::PreGame => {
                self.update_dynamic_buffers(display, time_step.time_delta)?;
                self.draw_scenary(frame, time_step, 0)?;
            }
            GameState::Game(round_counter) => {
                if round_counter.round > 0
                    || sound_ref.get_duration_ms("go".to_string())?
                        < round_counter.time_info.elapsed().as_millis() as f32
                {
                    //update position visulization create shots
                    self.handle_mask()?;
                    //hit test
                    self.hit_test(time_step)?;
                    //update vertex/index buffer particle_data
                    self.update_dynamic_buffers(display, time_step.time_delta)?;
                }

                //draw everything
                self.draw_scenary(frame, time_step, round_counter.round as usize)?;
                self.draw_skulls(frame, &params)?;
                self.draw_particles(frame, &params)?;

                //check for win condition
                if let Some(moon_d) = self.moon_data.as_mut() {
                    if moon_d.moon.life.current_value == 0_f32 {
                        let sound_ref_mut = self.sound.as_mut().ok_or("sound not intitialized")?;
                        if round_counter.round + 1 >= round_counter.max_round {
                            sound_ref_mut.stop_bgm();
                            *state = GameState::PostGame(RoundCounter::new(
                                round_counter.round + 1,
                                &self.settings,
                            ));
                            sound_ref_mut.play("finish", SoundType::Sfx)?;
                        } else {
                            *state = GameState::Intermission(round_counter);
                            moon_d.moon.heal(moon_d.moon.max_life);
                            sound_ref_mut.play("intermission", SoundType::Sfx)?;
                        };
                        self.kill_all_skulls()?;
                    }
                }
            }

            GameState::Intermission(round_counter) => {
                //just display moon healing press start key to continue
                self.update_dynamic_buffers(display, time_step.time_delta)?;
                if let Some(particles) = &mut self.particle_data {
                    for particle in particles.particles.iter_mut() {
                        particle.update();
                    }
                }
                self.draw_scenary(frame, time_step, (round_counter.round + 1) as usize)?;
                self.draw_particles(frame, &params)?;
            }
            GameState::PostGame(round_counter) => {
                let intro_over = sound_ref.get_duration_ms("finish".to_string())?
                    < round_counter.time_info.elapsed().as_millis() as f32;
                self.draw_scenary(frame, time_step, (round_counter.round) as usize)?;

                if intro_over {
                    self.draw_victory(frame)?;
                }
            }
        };
        Ok(())
    }

    fn key_event(&mut self, event: &Key) {
        let mut state = self.game_state.lock().unwrap();
        let mut play_start = || -> Result<(), Box<dyn std::error::Error>> {
            if let Some(sound_ref) = self.sound.as_mut() {
                sound_ref.play("go", SoundType::Sfx)?;
                sound_ref.start_bgm("bgm".to_string())?;
                Ok(())
            } else {
                Err("sound not initialized".into())
            }
        };

        if let Key::Character(val) = event.as_ref() {
            match &*state {
                GameState::PreGame => {
                    if val.to_lowercase() == self.settings.key_settings.start_key {
                        *state = GameState::Game(RoundCounter::new(0, &self.settings));
                        if let Err(err) = play_start() {
                            println!("Error in playing sound {}. Continuing", err)
                        }
                    }
                }
                GameState::Intermission(round_counter) => {
                    if val.to_lowercase() == self.settings.key_settings.start_key {
                        *state = GameState::Game(RoundCounter::new(
                            round_counter.round + 1,
                            &self.settings,
                        ));
                    }
                }
                _ => {} //can;t start game in current state
            }
        };

        match event.as_ref() {
            Key::Character(val) if val == self.settings.key_settings.normal_mode_key => {
                println!("set difficulty normal");
                self.difficultiy = self.settings.difficultiy_settings.normal;
            }
            Key::Character(val) if val == self.settings.key_settings.easy_mode_key => {
                println!("set difficulty easy");
                self.difficultiy = self.settings.difficultiy_settings.easy;
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

        if let Some(sound_ref) = self.sound.as_mut() {
            sound_ref.stop_bgm();
        };

        *self.game_state.lock().unwrap() = GameState::PreGame;
    }
}
