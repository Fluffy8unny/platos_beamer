use crate::types::BackgroundSubtractor;
use opencv::Result;
use opencv::bgsegm::{BackgroundSubtractorMOG, create_background_subtractor_mog};
use opencv::core::greater_than_mat_f64;
use opencv::core::{Mat, MatExpr, Ptr, Vector, split};
use opencv::prelude::*;
use opencv::video::calc_optical_flow_farneback;

#[derive(Debug, Clone, Copy)]
pub struct OfSettings {
    scales: i32,
    win_size: i32,
    iterations: i32,
    poly_n: i32,
    poly_sigma: f64,
    flags: i32,
}

impl OfSettings {
    pub fn default() -> OfSettings {
        OfSettings {
            scales: 3_i32,
            win_size: 15_i32,
            iterations: 3_i32,
            poly_n: 5_i32,
            poly_sigma: 1.2_f64,
            flags: 0_i32,
        }
    }
}

pub struct OfSubtractor {
    settings: OfSettings,
    prev_img: Option<Mat>,
}

impl OfSubtractor {
    pub fn new(settings: OfSettings) -> Result<OfSubtractor> {
        Ok(OfSubtractor {
            settings,
            prev_img: None,
        })
    }
}

impl BackgroundSubtractor for OfSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr> {
        let prev_img = match &self.prev_img {
            Some(prev) => &prev,
            None => &input_img,
        };

        let mut flow = Mat::default();
        calc_optical_flow_farneback(
            prev_img,
            &input_img,
            &mut flow,
            0.5,
            self.settings.scales,
            self.settings.win_size,
            self.settings.iterations,
            self.settings.poly_n,
            self.settings.poly_sigma,
            self.settings.flags,
        )?;
        let mut channels: Vector<Mat> = Vector::default();
        split(&flow, &mut channels)?;
        let dx_mag = channels.get(0)?;
        let dy_mag = channels.get(1)?;
        let magnitude = dx_mag.mul(&dx_mag, 1.0)? + dy_mag.mul(&dy_mag, 1.0)?;
        self.prev_img = Some(input_img);

        Ok(magnitude.into_result()?)
    }

    fn reset(&mut self, _background_img: Mat) {
        self.prev_img = None;
    }
}
