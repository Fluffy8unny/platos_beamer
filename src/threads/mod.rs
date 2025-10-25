mod bg_subtract;
mod camera;
mod util;

pub use bg_subtract::bg_subtract_pipeline;
pub use camera::{camera_thread, validate_camera};
pub use util::try_sending;
