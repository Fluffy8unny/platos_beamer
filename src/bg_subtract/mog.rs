use opencv::Result;
use opencv::bgsegm::{BackgroundSubtractorMOG, create_background_subtractor_mog};
use opencv::core::greater_than_mat_f64;
use opencv::core::{Mat, MatExpr, Ptr};
use opencv::prelude::*;

use serde::Deserialize;

use crate::types::BackgroundSubtractor;

#[derive(Deserialize, Clone, Copy)]
pub struct MogSettings {
    history: i32,
    mixtures: i32,
    background_ratio: f64,
    noise_sigma: f64,
    learning_rate: f64,
}

impl MogSettings {
    pub fn default() -> MogSettings {
        MogSettings {
            history: 250,
            mixtures: 5,
            background_ratio: 0.65,
            noise_sigma: 5.2,
            learning_rate: 0.05,
        }
    }
}

pub struct MogSubtractor {
    subtractor: Ptr<BackgroundSubtractorMOG>,
    settings: MogSettings,
}

fn mog_from_settings(
    MogSettings {
        history,
        mixtures,
        background_ratio,
        noise_sigma,
        learning_rate: _,
    }: MogSettings,
) -> Result<Ptr<BackgroundSubtractorMOG>> {
    create_background_subtractor_mog(history, mixtures, background_ratio, noise_sigma)
}

impl MogSubtractor {
    pub fn new(settings: MogSettings) -> Result<MogSubtractor> {
        let subtractor = mog_from_settings(settings)?;
        Ok(MogSubtractor {
            subtractor,
            settings,
        })
    }
}

impl BackgroundSubtractor for MogSubtractor {
    fn apply(&mut self, input_img: Mat) -> Result<MatExpr> {
        let mut mask = Mat::default();
        let res = self
            .subtractor
            .apply(&input_img, &mut mask, self.settings.learning_rate);
        res?;
        greater_than_mat_f64(&mask, 0.5)
    }

    fn reset(&mut self, _background_img: Mat) {
        if let Ok(subtractor) = mog_from_settings(self.settings) {
            self.subtractor = subtractor;
        } else {
            eprint!("could not reset mog background subtractor");
        }
    }
}
