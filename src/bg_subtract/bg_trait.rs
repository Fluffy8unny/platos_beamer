use opencv::Result;
use opencv::core::{Mat, MatExpr};

pub trait BackgroundSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr>;
    fn reset(&mut self, background_img: Mat);
}
