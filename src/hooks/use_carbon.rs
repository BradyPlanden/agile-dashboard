use crate::models::carbon::CarbonIntensity;
use crate::models::load_state::LoadState;
use crate::services::carbon_api::fetch_carbon_intensity;
use yew::prelude::*;

/// Hook that fetches and polls UK carbon intensity data.
#[hook]
pub fn use_carbon_intensity() -> LoadState<CarbonIntensity> {
    crate::models::load_state::use_polling(fetch_carbon_intensity)
}
