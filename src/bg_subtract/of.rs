use crate::types::BackgroundSubtractor;
use opencv::Result;
use opencv::core::{Mat, MatExpr, Vector, cart_to_polar, greater_than_mat_f64, split};
use opencv::imgproc::{COLOR_BGR2GRAY, cvt_color};
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

fn calc_flow_magnitude(flow: Mat) -> Result<Mat> {
    let mut channels: Vector<Mat> = Vector::default();
    split(&flow, &mut channels)?;

    let dx = channels.get(0)?;
    let dy = channels.get(1)?;

    let (mut magnitude, mut angle) = (Mat::default(), Mat::default());
    cart_to_polar(&dx, &dy, &mut magnitude, &mut angle, false)?;

    Ok(magnitude)
}

impl BackgroundSubtractor for OfSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr> {
        let mut gray_input = Mat::default();
        cvt_color(
            &input_img,
            &mut gray_input,
            COLOR_BGR2GRAY,
            1,
            opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let prev_img = match &self.prev_img {
            Some(prev) => prev,
            None => &gray_input,
        };

        let mut flow = Mat::default();
        calc_optical_flow_farneback(
            prev_img,
            &gray_input,
            &mut flow,
            0.5,
            self.settings.scales,
            self.settings.win_size,
            self.settings.iterations,
            self.settings.poly_n,
            self.settings.poly_sigma,
            self.settings.flags,
        )?;
        self.prev_img = Some(gray_input);
        let magnitude = calc_flow_magnitude(flow)?;
        greater_than_mat_f64(&magnitude, 5.0)
    }

    fn reset(&mut self, _background_img: Mat) {
        self.prev_img = None;
    }
}
