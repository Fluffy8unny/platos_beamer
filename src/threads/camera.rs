use opencv::prelude::*;
use opencv::videoio::VideoCapture;
use opencv::{Error, Result, videoio};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::SystemTime;

use crate::threads::try_sending;
use crate::types::thread_types::*;

fn get_camera(camera_index: i32) -> Result<VideoCapture> {
    videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)
}

pub fn validate_camera(camera_index: i32) -> Result<()> {
    let mut cam = get_camera(camera_index)?;
    let res = cam.open(camera_index, videoio::CAP_ANY)?;
    let _ = cam.release();

    match res {
        true => Ok(()),
        false => Err(Error {
            code: -10,
            message: "could not open camera".to_string(),
        }),
    }
}

pub fn camera_thread(
    camera_controller_queue: Receiver<CameraMessage>,
    image_queue: SyncSender<CameraResult>,
    camera_index: i32,
) -> Result<()> {
    let mut cam = get_camera(camera_index)?;
    loop {
        match camera_controller_queue.recv() {
            Ok(msg) => match msg {
                CameraMessage::Quit => {
                    println!("Quitting grabber gracefully");
                    return Ok(());
                }
                CameraMessage::GetImage => {
                    let mut frame = Mat::default();
                    let camera_result = cam.read(&mut frame);
                    let image = match camera_result {
                        Err(e) => Err(e),
                        Ok(..) => Ok(frame),
                    };

                    try_sending(
                        &image_queue,
                        CameraResult {
                            data: image,
                            timestamp: SystemTime::now(),
                        },
                        "grabber thread",
                        "image_queue",
                    );
                }
            },
            Err(error) => {
                eprintln!("receiver error (Camera Thread, camera_controller_queue): {error}")
            }
        }
    }
}
