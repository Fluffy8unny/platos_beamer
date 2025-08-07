use opencv::core::{MatExpr, MatExprResult, Vector, absdiff, greater_than_mat_f64, split};
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

#[derive(Debug)]
struct CameraResult {
    data: Result<Mat>,
    timestamp: SystemTime,
}

fn try_sending<T>(sender: &SyncSender<T>, message: T, thread_name: &str, queue_name: &str) {
    if let Err(error) = sender.send(message) {
        eprintln!("Send error.{thread_name}, {queue_name}. {error}")
    }
}

fn camera_thread(
    camera_controller_queue: Receiver<CameraMessage>,
    image_queue: SyncSender<CameraResult>,
    camera_index: i32,
) -> Result<()> {
    let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
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

fn compute_resulting_image(
    image: CameraResult,
    reference: &Result<Mat>,
    compute_fn: fn(Mat, Mat) -> Result<MatExpr>,
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
    println!("{:?}", image.timestamp);
    let res = compute_fn(input_image, reference_image);

    match res {
        Ok(res) => res.to_mat(),
        Err(..) => Err(opencv::Error {
            code: 2,
            message: "Math error in compute function".to_string(),
        }),
    }
}

fn pipeline_thread(
    camera_control_queue: SyncSender<CameraMessage>,
    image_grabbing_queue: Receiver<CameraResult>,
    pipeline_control_queue: Receiver<PipelineMessage>,
    result_queue: SyncSender<Result<Mat>>,
    compute_fn: fn(Mat, Mat) -> Result<MatExpr>,
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
                    //discard queried image. If we don't query before we know what happens we waste
                    //time, but here we have to discard one
                    let _ = image_grabbing_queue.recv();
                    println!("Quitting pipeline gracefully");
                    return Ok(());
                }
                PipelineMessage::GenerateImage => match image_grabbing_queue.recv() {
                    Err(error) => {
                        eprintln!("receiver error (Pipeline thread, image_grabbing_queue) {error}")
                    }
                    Ok(result) => {
                        let output_image =
                            compute_resulting_image(result, &reference_image, compute_fn);
                        try_sending(
                            &result_queue,
                            output_image,
                            "pipeline thread",
                            "result queue",
                        );
                    }
                },
                PipelineMessage::SetReference => match image_grabbing_queue.recv() {
                    Err(error) => {
                        eprintln!("receiver error (Pipeline thread, image_grabbing_queue) {error}")
                    }
                    Ok(result) => {
                        reference_image = match result.data {
                            Ok(image_data) => Ok(image_data),
                            Err(error) => {
                                eprintln!(
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
                    eprintln!("Window thread reuslt_queue. Received frame is error {error}")
                }
            },
            Err(error) => eprintln!(
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
            println!("Quitting window gracefully");
            return Ok(());
        }
        if key == 878255 {
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
            |img, ref_img| {
                let mut res = Mat::default();
                let _ = absdiff(&img, &ref_img, &mut res);

                let mut channels: Vector<Mat> = Vector::default();
                let _ = split(&res, &mut channels);
                let num_channels = channels.len();

                let init = channels.get(0);
                let acc = channels
                    .iter()
                    .skip(1)
                    .fold(init, |acc, m| (acc? + (m)).into_result()?.to_mat());
                let acc_res = acc?;

                greater_than_mat_f64(&acc_res, (num_channels as f64) * 50_f64)
            },
        )
    });
    let window_handle =
        thread::spawn(move || window_thread(pipeline_control_sender, result_receiver));

    [window_handle, pipeline_handle, grab_handle].map(|t| {
        let _res = t.join().unwrap();
    });
    Ok(())
}
