use opencv::prelude::*;
use opencv::{Result, highgui, videoio};

use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
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
    //Gray8,
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

fn compute_resulting_image(
    image: CameraResult,
    reference: &Result<Mat>,
    compute_fn: fn(Mat, Mat) -> Result<Mat>,
) -> Result<Mat> {
    let input_image = image.data?;
    let reference_image = match reference {
        Ok(res) => res.clone(),
        Err(_) => {
            return Err(opencv::Error {
                code: 1,
                message: "Reference image not set".to_string(),
            });
        }
    };
    println!("{:?} {:?}", image.format, image.timestamp);
    compute_fn(input_image, reference_image)
}

fn try_sending<T>(sender: &SyncSender<T>, message: T, thread_name: &str, queue_name: &str) {
    if let Err(error) = sender.send(message) {
        eprint!("Send error.{thread_name}, {queue_name}. {error}")
    }
}

fn pipeline_thread(
    camera_control_queue: SyncSender<CameraMessage>,
    image_grabbing_queue: Receiver<CameraResult>,
    pipeline_control_queue: Receiver<PipelineMessage>,
    result_queue: SyncSender<Result<Mat>>,
    compute_fn: fn(Mat, Mat) -> Result<Mat>,
) -> Result<()> {
    let mut reference_image: Result<Mat> = Err(opencv::Error {
        code: 1,
        message: "Reference image not set yet".to_string(),
    });
    loop {
        //always query an image
        try_sending(
            &camera_control_queue,
            CameraMessage::GetImage,
            "pipeline_thread",
            "camera control queue",
        );

        //wait for msg from the window thread
        match pipeline_control_queue.recv() {
            Ok(msg) => match msg {
                PipelineMessage::Quit => {
                    try_sending(
                        &camera_control_queue,
                        CameraMessage::Quit,
                        "pipeline thread",
                        "camera control queue",
                    );
                    return Ok(());
                }
                PipelineMessage::GenerateImage => match image_grabbing_queue.recv() {
                    Err(error) => {
                        eprint!("receiver error (Pipeline thread, image_grabbing_queue) {error}")
                    }
                    Ok(result) => {
                        let output_image =
                            compute_resulting_image(result, &reference_image, compute_fn);
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
                            Ok(image_data) => Ok(image_data),
                            Err(error) => {
                                eprint!(
                                    "receiver error (Pipeline thread, image_grabbing_queue. Could not set reference image.) {error}"
                                );
                                //return current reference image
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

//todo replace with something but opencv, this is just for testing purposes
fn window_thread(
    pipeline_control_queue: SyncSender<PipelineMessage>,
    result_queue: Receiver<Result<Mat>>,
) -> Result<()> {
    let window = "platos beamer";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    //init pipeline
    try_sending(
        &pipeline_control_queue,
        PipelineMessage::SetReference,
        "window thread",
        "pipeline_control_queue",
    );

    loop {
        try_sending(
            &pipeline_control_queue,
            PipelineMessage::GenerateImage,
            "window thread",
            "pipeline_control_queue",
        );
        match result_queue.recv() {
            Ok(result) => match result {
                Ok(mat) => highgui::imshow(window, &mat)?,
                Err(error) => {
                    eprint!("Window thread reuslt_queue. Received frame is error {error}")
                }
            },
            Err(error) => eprint!(
                "Receiver error(Window thread, result_queue. COuld not receive frame {error})"
            ),
        }

        let key = highgui::wait_key(10)?;
        if key > 0 && key != 255 {
            try_sending(
                &pipeline_control_queue,
                PipelineMessage::Quit,
                "window_thread",
                "pipeline_control_queue",
            );
            return Ok(());
        }
        if key == 255 {
            try_sending(
                &pipeline_control_queue,
                PipelineMessage::SetReference,
                "window_thread",
                "pipeline_control_queue",
            );
            return Ok(());
        }
    }
}
fn main() -> Result<()> {
    let camera_index = 0;

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
        pipeline_thread(
            camera_control_sender,
            image_receiver,
            pipeline_control_receiver,
            result_sender,
            |img, _ref_img| Ok(img),
        )
    });
    let window_handle =
        thread::spawn(move || window_thread(pipeline_control_sender, result_receiver));

    [grab_handle, pipeline_handle, window_handle].map(|t| {
        let _res = t.join().unwrap();
    });
    Ok(())
}
