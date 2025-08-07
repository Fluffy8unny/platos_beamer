mod bg_subtract;
mod threads;
mod types;

use crate::threads::{bg_subtract_pipeline, camera_thread, display_window_thread};
use crate::types::thread_types::{CameraMessage, CameraResult, PipelineMessage};

use opencv::Result;
use opencv::core::{MatExpr, Vector, absdiff, greater_than_mat_f64, split};
use opencv::prelude::*;

use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;

fn naive_background_removal(img: Mat, ref_img: Mat) -> Result<MatExpr> {
    let mut res = Mat::default();
    let _ = absdiff(&img, &ref_img, &mut res);

    let mut channels: Vector<Mat> = Vector::default();
    let _ = split(&res, &mut channels);

    let init = channels.get(0);
    let acc = channels
        .iter()
        .skip(1)
        .fold(init, |acc, m| (acc? + (m)).into_result()?.to_mat());
    let acc_res = acc?;

    greater_than_mat_f64(&acc_res, 100_f64)
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
        bg_subtract_pipeline(
            camera_control_sender,
            image_receiver,
            pipeline_control_receiver,
            result_sender,
            naive_background_removal,
        )
    });
    let window_handle =
        thread::spawn(move || display_window_thread(pipeline_control_sender, result_receiver));

    [window_handle, pipeline_handle, grab_handle].map(|t| {
        let _res = t.join().unwrap();
    });
    Ok(())
}
