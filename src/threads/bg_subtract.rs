use opencv::Result;
use opencv::prelude::*;

use std::sync::mpsc::{Receiver, SyncSender};
use std::time::SystemTime;

use crate::threads::try_sending;
use crate::types::BackgroundSubtractor;
use crate::types::thread_types::*;

fn compute_resulting_image(
    image: CameraResult,
    subtractor: &mut Box<dyn BackgroundSubtractor>,
) -> Result<Mat> {
    let input_image = image.data?;
    println!("{:?}", image.timestamp);
    let res = subtractor.apply(input_image);

    match res {
        Ok(res) => res.to_mat(),
        Err(..) => Err(opencv::Error {
            code: 2,
            message: "Math error in compute function".to_string(),
        }),
    }
}

pub fn bg_subtract_pipeline(
    camera_control_queue: SyncSender<CameraMessage>,
    image_grabbing_queue: Receiver<CameraResult>,
    pipeline_control_queue: Receiver<PipelineMessage>,
    result_queue: SyncSender<BackgroundResult>,
    bg_subtractor: Box<dyn BackgroundSubtractor>,
) -> Result<()> {
    let mut subtractor = bg_subtractor;
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
                        let output_image = compute_resulting_image(result, &mut subtractor);
                        try_sending(
                            &result_queue,
                            BackgroundResult {
                                data: output_image,
                                timestamp: SystemTime::now(),
                            },
                            "pipeline thread",
                            "result queue",
                        );
                    }
                },
                PipelineMessage::SetReference => match image_grabbing_queue.recv() {
                    Err(error) => {
                        eprintln!("receiver error (Pipeline thread, image_grabbing_queue) {error}")
                    }
                    Ok(result) => match result.data {
                        Ok(image_data) => subtractor.reset(image_data),
                        Err(error) => eprintln!(
                            "receiver error (Pipeline thread, image_grabbing_queue. Could not set reference image.) {error}"
                        ),
                    },
                },
            },
            Err(error) => {
                eprintln!("receiver error (Pipeline Thread, pipeline_control_queue): {error}")
            }
        }
    }
}
