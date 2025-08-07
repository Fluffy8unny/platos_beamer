use opencv::prelude::*;
use opencv::{Result, highgui};

use std::sync::mpsc::{Receiver, SyncSender};

use crate::threads::try_sending;
use crate::types::thread_types::*;

//todo replace with something but opencv, this is just for testing purposes
pub fn display_window_thread(
    pipeline_control_queue: SyncSender<PipelineMessage>,
    result_queue: Receiver<Result<Mat>>,
) -> Result<()> {
    let window = "platos beamer";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    //init pipeline, so defaults will be available
    try_sending(
        &pipeline_control_queue,
        PipelineMessage::SetReference,
        "window thread",
        "pipeline_control_queue",
    );

    loop {
        //ask for image every frame. This way it'll be ready asap,
        //since we inited b4 the loop.
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
