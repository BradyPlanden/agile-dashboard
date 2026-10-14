use yew::prelude::*;

use crate::config::POLL_INTERVAL_MS;
use crate::hooks::use_polling::{Polled, use_polling};
use crate::models::rates::Rates;
use crate::models::region::Region;
use crate::services::octopus::fetch_agile_rates;

/// Agile rates for today and tomorrow in `region`, refreshed periodically.
#[hook]
pub fn use_rates(region: Region) -> Polled<Rates> {
    use_polling(region, POLL_INTERVAL_MS, fetch_agile_rates)
}
