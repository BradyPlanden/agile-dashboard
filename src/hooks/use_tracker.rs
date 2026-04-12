use crate::models::load_state::LoadState;
use crate::models::rates::TrackerRates;
use crate::services::api::{Region, fetch_tracker_rates_for_region};
use yew::prelude::*;

/// Hook that fetches and polls Tracker rates for a given region.
#[hook]
pub fn use_tracker_rates(region: Region) -> LoadState<TrackerRates> {
    let region_for_fetch = region;
    crate::models::load_state::use_polling_with_key(
        move || fetch_tracker_rates_for_region(region_for_fetch),
        region,
    )
}
