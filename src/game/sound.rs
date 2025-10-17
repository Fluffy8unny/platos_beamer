use crate::config::SoundConfig;
use rodio::{
    Decoder, OutputStream, OutputStreamBuilder, Sink,
    source::{Buffered, Repeat, Source, Stoppable},
};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub type SoundSource = Buffered<Decoder<BufReader<File>>>;
pub type SoundSourceResult = Result<SoundSource, Box<dyn std::error::Error>>;
pub enum SoundType {
    Sfx,
    Music,
}

pub struct AudioHandler {
    stream_handle: OutputStream,
    _sink: Sink, //needs to have the same lifetime as stream_handle
    sounds: HashMap<String, SoundSourceResult>,
    config: SoundConfig,
    background_music: Option<Sink>,
}

fn load_sound_data(path: &str) -> SoundSourceResult {
    let file = File::open(path)?;
    let buff_reader = BufReader::new(file);
    Ok(Decoder::new(buff_reader)?.buffered())
}

impl AudioHandler {
    pub fn new(
        sounds: Vec<(String, String)>,
        config: SoundConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        let sounds: HashMap<String, SoundSourceResult> = sounds
            .into_iter()
            .map(|(name, path)| -> (String, SoundSourceResult) { (name, load_sound_data(&path)) })
            .collect();
        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        Ok(AudioHandler {
            stream_handle,
            _sink: sink,
            sounds,
            background_music: None,
            config,
        })
    }

    pub fn stop_bgm(&mut self) {
        if let Some(sink) = self.background_music.as_mut() {
            sink.stop();
        }
    }

    pub fn start_bgm(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        self.stop_bgm();
        let repeating_source = match self
            .sounds
            .get(&name)
            .ok_or(format!("sound {:?} not found", name))?
            .as_ref()
        {
            Ok(res) => res
                .clone()
                .repeat_infinite()
                .stoppable()
                .amplify_normalized(self.get_volume(SoundType::Music)),
            Err(_err) => return Err(format!("sound not found {:?}", name).into()),
        };
        let sink = rodio::Sink::connect_new(self.stream_handle.mixer());
        sink.append(repeating_source);
        self.background_music = Some(sink);
        Ok(())
    }

    fn get_volume(&self, sound_type: SoundType) -> f32 {
        let amp = match sound_type {
            SoundType::Sfx => self.config.sfx_volume,
            SoundType::Music => self.config.music_volume,
        };
        amp * self.config.master_volume
    }

    pub fn play(
        &self,
        name: &str,
        sound_type: SoundType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.sounds.get(name).ok_or("sound not found")?.as_ref() {
            Ok(sound_data) => {
                let buffered_source = sound_data.clone();
                self.stream_handle
                    .mixer()
                    .add(buffered_source.amplify_normalized(self.get_volume(sound_type)));
                Ok(())
            }
            Err(_err) => Err(format!("Sound not found {:?}", name).into()),
        }
    }

    pub fn add_sound(&mut self, name: String, path: String) {
        let sound_source = load_sound_data(&path);
        self.sounds.insert(name, sound_source);
    }
}
