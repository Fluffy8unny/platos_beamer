use opencv::Result;
use opencv::core::{Mat, MatExpr, Vector, absdiff, greater_than_mat_f64, split};
use opencv::prelude::*;

use crate::bg_subtract::BackgroundSubtractor;

pub struct NaiveSubtractor {
    pub background_approximation: Mat,
}

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

impl BackgroundSubtractor for NaiveSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr> {
        naive_background_removal(input_img, self.background_approximation.clone())
    }

    fn reset(&mut self, background_img: Mat) {
        self.background_approximation = background_img;
    }
}
