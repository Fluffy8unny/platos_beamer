mod bg_trait;
mod mog;
mod naive;

pub use bg_trait::BackgroundSubtractor;
pub use mog::{MogSettings, MogSubtractor};
pub use naive::{NaiveSettings, NaiveSubtractor};

use serde::Deserialize;

#[derive(Deserialize)]
pub enum SubtractorType {
    Naive,
    Mog,
}
