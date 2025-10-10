use crate::{
    config::PlatoConfig, display::display_window::DisplayType, display::timestep::TimeStep,
};

use glium::winit::keyboard::Key;
use opencv::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub enum GameType {
    IdentityGame,
    SkullGame,
    CalibrationGame,
}

use rodio::mixer;
use rodio::mixer::{Mixer, MixerSource};
use rodio::{OutputStream, Sink};

struct SoundHandler {
    controller: Mixer,
    mixer: MixerSource,
    stream_handle: OutputStream,
    sink: Sink,
}

impl SoundHandler {
    fn new() -> Result<SoundHandler, Box<dyn std::error::Error>> {
        let (controller, mixer) = mixer::mixer(2, 44);
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        Ok(SoundHandler {
            controller,
            mixer,
            stream_handle,
            sink,
        })
    }
}

pub trait GameTrait {
    fn init(
        &mut self,
        display: &DisplayType,
        config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn update(
        &mut self,
        image: &Mat,
        mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        display: &DisplayType,
        timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn key_event(&mut self, event: &Key) {}

    fn reset(&mut self) {}
}
