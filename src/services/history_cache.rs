//! Local cache for the price-history banner, so it costs no network after the first load of the day.

use chrono::{DateTime, Utc};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use crate::models::error::AppError;
use crate::models::rates::Rates;
use crate::models::region::Region;
use crate::services::octopus::{fetch_agile_history, history_window};
use crate::utils::log;

const KEY: &str = "historical_rates";
const VERSION: u8 = 2;

#[derive(Serialize, Deserialize)]
struct Entry {
    version: u8,
    region: Region,
    window_end: DateTime<Utc>,
    rates: Rates,
}

/// Agile history for `region`, from the cache when it already covers today's window.
///
/// Past prices never change, so an entry stays valid until the window moves at London midnight.
pub async fn load_history(region: Region) -> Result<Rates, AppError> {
    let now = Utc::now();
    let (_, window_end) = history_window(now);

    if let Some(rates) = read(region, window_end) {
        return Ok(rates);
    }

    let rates = fetch_agile_history(region, now).await?;
    write(&Entry {
        version: VERSION,
        region,
        window_end,
        rates: rates.clone(),
    });
    Ok(rates)
}

fn read(region: Region, window_end: DateTime<Utc>) -> Option<Rates> {
    let entry: Entry = LocalStorage::get(KEY).ok()?;
    is_current(&entry, region, window_end).then_some(entry.rates)
}

fn write(entry: &Entry) {
    if let Err(error) = LocalStorage::set(KEY, entry) {
        log::warn(&format!("Failed to cache price history: {error}"));
    }
}

fn is_current(entry: &Entry, region: Region, window_end: DateTime<Utc>) -> bool {
    entry.version == VERSION && entry.region == region && entry.window_end == window_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn entry(region: Region, window_end: DateTime<Utc>) -> Entry {
        Entry {
            version: VERSION,
            region,
            window_end,
            rates: Rates::default(),
        }
    }

    #[test]
    fn entry_is_current_only_for_the_same_region_and_window() {
        let end = Utc.with_ymd_and_hms(2026, 4, 11, 23, 0, 0).unwrap();
        let next_end = Utc.with_ymd_and_hms(2026, 4, 12, 23, 0, 0).unwrap();

        assert!(is_current(&entry(Region::C, end), Region::C, end));
        assert!(!is_current(&entry(Region::C, end), Region::M, end));
        assert!(!is_current(&entry(Region::C, end), Region::C, next_end));
    }

    #[test]
    fn entry_from_an_older_format_is_not_current() {
        let end = Utc.with_ymd_and_hms(2026, 4, 11, 23, 0, 0).unwrap();
        let old = Entry {
            version: VERSION - 1,
            ..entry(Region::C, end)
        };
        assert!(!is_current(&old, Region::C, end));
    }
}
