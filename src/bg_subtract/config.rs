use serde::Deserialize;

use crate::bg_subtract::mog::MogSettings;
use crate::bg_subtract::naive::NaiveSettings;
use crate::bg_subtract::of::OfSettings;
use crate::bg_subtract::test::TestSettings;

#[derive(Deserialize, Clone)]
pub struct BGSubtracSettings {
    pub mog_settings: MogSettings,
    pub naive_settings: NaiveSettings,
    pub of_settings: OfSettings,
    pub test_settings: TestSettings,
}
