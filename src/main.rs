use opencv::imgproc::ColorConversionCodes;
use opencv::prelude::*;
use opencv::{Result, highgui, videoio};

use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::thread;
use std::time::SystemTime;

enum CameraMessage {
    GetImage,
    Quit,
}
enum PipelineMessage {
    GenerateImage,
    SetReference,
    Quit,
}
#[derive(Debug, Clone, Copy)]
enum ImageColor {
    Rgb8,
    Gray8,
}
#[derive(Debug)]
struct CameraResult {
    data: Result<Mat>,
    format: ImageColor,
    timestamp: SystemTime,
}

fn camera_thread(
    camera_controller_queue: Receiver<CameraMessage>,
    image_queue: SyncSender<CameraResult>,
    camera_index: i32,
) -> Result<()> {
    let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
    let camera_format = ImageColor::Rgb8;
    loop {
        match camera_controller_queue.recv() {
            Ok(msg) => match msg {
                CameraMessage::Quit => return Ok(()),
                CameraMessage::GetImage => {
                    let mut frame = Mat::default();
                    let camera_result = cam.read(&mut frame);
                    let image = match camera_result {
                        Err(e) => Err(e),
                        Ok(..) => Ok(frame),
                    };
                    match image_queue.send(CameraResult {
                        data: image,
                        format: camera_format,
                        timestamp: SystemTime::now(),
                    }) {
                        Ok(..) => {}
                        Err(error) => eprint!("sender error {error}"),
                    };
                }
            },
            Err(error) => {
                eprintln!("receiver error (Camera Thread, camera_controller_queue): {error}")
            }
        }
    }
}
fn compute_resulting_image(image: CameraResult, reference: &Option<Mat>) -> Result<Mat> {
    Err("not implemented yet")
}

fn pipeline_thread(
    camera_control_queue: SyncSender<CameraMessage>,
    image_grabbing_queue: Receiver<CameraResult>,
    pipeline_control_queue: Receiver<PipelineMessage>,
    result_queue: SyncSender<Result<Mat>>,
) -> Result<()> {
    let mut reference_image: Option<Mat> = None;
    loop {
        //always query an image
        match camera_control_queue.send(CameraMessage::GetImage) {
            Err(error) => eprint!(
                "Send error.Pipeline thread, camera control queue. Could not query image.{error}"
            ),
            Ok(..) => {}
        }

        //wait for msg from the window thread
        match pipeline_control_queue.recv() {
            Ok(msg) => match msg {
                PipelineMessage::Quit => {
                    match camera_control_queue.send(CameraMessage::Quit) {
                        Err(error) => eprint!(
                            "Pipeline Thread, camera_control_queue. Send error, could not send quit message. {error}"
                        ),
                        Ok(..) => {}
                    };
                    return Ok(());
                }
                PipelineMessage::GenerateImage => match image_grabbing_queue.recv() {
                    Err(error) => {
                        eprint!("receiver error (Pipeline thread, image_grabbing_queue) {error}")
                    }
                    Ok(result) => {
                        let output_image = compute_resulting_image(result, &reference_image);
                        match result_queue.send(output_image) {
                            Ok(..) => {}
                            Err(error) => {
                                eprint!("Sending error(Pipeline thread,result_queue) {error}")
                            }
                        }
                    }
                },
                PipelineMessage::SetReference => match image_grabbing_queue.recv() {
                    Err(error) => {
                        eprint!("receiver error (Pipeline thread, image_grabbing_queue) {error}")
                    }
                    Ok(result) => {
                        reference_image = match result.data {
                            Ok(image_data) => Some(image_data),
                            Err(error) => {
                                eprint!(
                                    "receiver error (Pipeline thread, image_grabbing_queue. Could not set reference image.) {error}"
                                );
                                reference_image
                            }
                        }
                    }
                },
            },
            Err(error) => {
                eprintln!("receiver error (Pipeline Thread, pipeline_control_queue): {error}")
            }
        }
    }
}

fn main() -> Result<()> {
    let camera_index = 0;
    let window = "video capture";

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

    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
    let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?; // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!")
    }
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        if frame.size()?.width > 0 {
            highgui::imshow(window, &frame)?;
        }
        let key = highgui::wait_key(10)?;
        if key > 0 && key != 255 {
            break;
        }
    }
    Ok(())
}
