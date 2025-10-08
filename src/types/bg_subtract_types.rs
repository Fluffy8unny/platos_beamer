use opencv::Result;
use opencv::core::{Mat, MatExpr};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub enum SubtractorType {
    Naive,
    Mog,
    OpticalFlow,
    Test,
}

pub trait BackgroundSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr>;
    fn reset(&mut self, background_img: Mat);
}
