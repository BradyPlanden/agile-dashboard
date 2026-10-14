use yew::prelude::*;

use crate::config::HISTORY_POLL_INTERVAL_MS;
use crate::hooks::use_polling::{Polled, use_polling};
use crate::models::rates::Rates;
use crate::models::region::Region;
use crate::services::history_cache::load_history;

/// The last 31 days of Agile rates in `region`, served from a daily local cache.
#[hook]
pub fn use_historical_rates(region: Region) -> Polled<Rates> {
    use_polling(region, HISTORY_POLL_INTERVAL_MS, load_history)
}
