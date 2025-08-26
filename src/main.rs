mod bg_subtract;
mod config;
mod display;
mod threads;
mod types;

use crate::bg_subtract::{MogSettings, MogSubtractor, NaiveSettings, NaiveSubtractor};
use crate::config::load_config;
use crate::display::start_display;
use crate::threads::{bg_subtract_pipeline, camera_thread, validate_camera};
use crate::types::{
    BackgroundSubtractor, CameraMessage, CameraResult, PipelineMessage, SubtractorType,
};
use opencv::prelude::*;
use opencv::{Error, Result};

use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;

fn create_bg_selector(selected_type: SubtractorType) -> Result<Box<dyn BackgroundSubtractor>> {
    Ok(match selected_type {
        SubtractorType::Mog => {
            Box::new(MogSubtractor::new(MogSettings::default())?) as Box<dyn BackgroundSubtractor>
        }
        SubtractorType::Naive => Box::new(NaiveSubtractor {
            background_approximation: Mat::default(),
            settings: NaiveSettings::default(),
        }) as Box<dyn BackgroundSubtractor>,
    })
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = load_config("config.toml")?;
    let camera_index = config.camera_config.device_index;
    print!("{:?}", camera_index);
    let selected_type = config.background_subtractor_config.subtractor_type;

    if validate_camera(camera_index).is_err() {
        eprintln!("could not find camera at device idx {}", camera_index);
        return Err(Box::new(Error::new(2, "could not open camera")));
    }

    let (image_sender, image_receiver): (SyncSender<CameraResult>, Receiver<CameraResult>) =
        sync_channel(1);
    let (camera_control_sender, camera_control_receiver): (
        SyncSender<CameraMessage>,
        Receiver<CameraMessage>,
    ) = sync_channel(1);
    let (pipeline_control_sender, pipeline_control_receiver): (
        SyncSender<PipelineMessage>,
        Receiver<PipelineMessage>,
    ) = sync_channel(1);
    let (result_sender, result_receiver): (SyncSender<Result<Mat>>, Receiver<Result<Mat>>) =
        sync_channel(1);

    let grab_handle =
        thread::spawn(move || camera_thread(camera_control_receiver, image_sender, camera_index));

    let pipeline_handle = thread::spawn(move || {
        bg_subtract_pipeline(
            camera_control_sender,
            image_receiver,
            pipeline_control_receiver,
            result_sender,
            create_bg_selector(selected_type)?,
        )
    });

    match (start_display(pipeline_control_sender, result_receiver)) {
        Ok(_) => {
            println!("shutting down other threads gracefully:");
            [pipeline_handle, grab_handle].map(|t| {
                let _res = t.join().unwrap();
            });
        }
        Err(err) => eprint!("everything is fucked {}", err),
    };
    Ok(())
}
