use crate::models::load_state::LoadState;
use crate::models::rates::Rates;
use crate::services::api::{Region, fetch_rates_for_region};
use yew::prelude::*;

/// Hook that fetches and polls Agile rates for a given region.
#[hook]
pub fn use_rates(region: Region) -> LoadState<Rates> {
    let region_for_fetch = region;
    crate::models::load_state::use_polling_with_key(
        move || fetch_rates_for_region(region_for_fetch),
        region,
    )
}
