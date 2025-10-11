use rodio::{
    source::{Buffered, Source},
    Decoder, OutputStream, OutputStreamBuilder, Sink,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub type SoundSource = Buffered<Decoder<BufReader<File>>>;
pub type SoundSourceResult = Result<SoundSource, Box<dyn std::error::Error>>;

pub struct AudioHandler {
    stream_handle: OutputStream,
    _sink: Sink, //needs to have the same lifetime as stream_handle
    sounds: HashMap<String, SoundSourceResult>,
}

fn load_sound_data(path: &str) -> SoundSourceResult {
    let file = File::open(path)?;
    let buff_reader = BufReader::new(file);
    Ok(Decoder::new(buff_reader)?.buffered())
}

impl AudioHandler {
    pub fn new(sounds: Vec<(String, String)>) -> Result<Self, Box<dyn std::error::Error>> {
        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        let sounds: HashMap<String, SoundSourceResult> = sounds
            .into_iter()
            .map(|(name, path)| -> (String, SoundSourceResult) { (name, load_sound_data(&path)) })
            .collect();
        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        Ok(AudioHandler {
            stream_handle,
            _sink:sink,
            sounds,
        })
    }

    pub fn play(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.sounds.get(name).ok_or("sound not found")?.as_ref() {
            Ok(sound_data) => {
                let buffered_source = sound_data.clone();
                self.stream_handle.mixer().add(buffered_source);
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
