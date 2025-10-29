use opencv::Result;
use opencv::core::{CV_8U, Mat, MatExpr, Rect_, Scalar};
use opencv::imgproc;
use opencv::prelude::*;

use crate::types::BackgroundSubtractor;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TestSettings {
    test_box_pos: (i32, i32),
    test_box_size: (i32, i32),
}

pub struct TestSubtractor {
    pub settings: TestSettings,
}

impl BackgroundSubtractor for TestSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr> {
        let mut res = Mat::zeros(input_img.rows(), input_img.cols(), CV_8U)?.to_mat()?;
        //let rec = Rect_{ x:0,y:0,width:200,height:200 };
        imgproc::rectangle(
            &mut res,
            Rect_::new(
                self.settings.test_box_pos.0,
                self.settings.test_box_pos.1,
                self.settings.test_box_size.0,
                self.settings.test_box_size.1,
            ),
            Scalar::from(255.0),
            -1,
            imgproc::LINE_AA,
            0,
        )?;
        MatExpr::from_mat(&res)
    }

    fn reset(&mut self, _background_img: Mat) {}
}
