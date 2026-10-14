use chrono::NaiveDate;
use yew::prelude::*;

use crate::config::HISTORY_POLL_INTERVAL_MS;
use crate::hooks::use_polling::{Polled, use_polling};
use crate::models::rates::Rates;
use crate::models::region::Region;
use crate::services::history_cache::load_history;

/// The 31 days of Agile rates in `region` before `today` (London time), served from a daily
/// local cache.
///
/// `today` is part of the polling key, so the window moves as soon as the date changes at
/// midnight rather than at the next scheduled poll.
#[hook]
pub fn use_historical_rates(region: Region, today: NaiveDate) -> Polled<Rates> {
    use_polling(
        (region, today),
        HISTORY_POLL_INTERVAL_MS,
        |(region, today)| load_history(region, today),
    )
}
