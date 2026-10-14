use yew::prelude::*;

use crate::config::POLL_INTERVAL_MS;
use crate::hooks::use_polling::{Polled, use_polling};
use crate::models::carbon::CarbonIntensity;
use crate::services::carbon::fetch_carbon_intensity;

/// GB grid carbon intensity, refreshed periodically.
#[hook]
pub fn use_carbon_intensity() -> Polled<CarbonIntensity> {
    use_polling((), POLL_INTERVAL_MS, |()| fetch_carbon_intensity())
}
