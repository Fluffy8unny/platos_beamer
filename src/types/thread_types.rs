use opencv::Result;
use opencv::core::Mat;
use std::time::SystemTime;

pub enum CameraMessage {
    GetImage,
    Quit,
}

pub enum PipelineMessage {
    GenerateImage,
    SetReference,
    Quit,
}

#[derive(Debug)]
pub struct CameraResult {
    pub data: Result<Mat>,
    pub timestamp: SystemTime,
}
