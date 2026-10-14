//! Local cache for the price-history banner, so it costs no network after the first load of the day.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use crate::models::error::AppError;
use crate::models::rates::Rates;
use crate::models::region::Region;
use crate::services::octopus::{fetch_agile_history, history_window};
use crate::utils::log;

const KEY: &str = "historical_rates";
const VERSION: u8 = 3;

/// The history of every region viewed since the window last moved, so switching back to a
/// region costs nothing.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Cache {
    version: u8,
    window_end: DateTime<Utc>,
    regions: BTreeMap<Region, Rates>,
}

impl Cache {
    const fn empty(window_end: DateTime<Utc>) -> Self {
        Self {
            version: VERSION,
            window_end,
            regions: BTreeMap::new(),
        }
    }

    /// This cache if it holds the window ending at `window_end`, otherwise an empty one.
    fn for_window(self, window_end: DateTime<Utc>) -> Self {
        if self.version == VERSION && self.window_end == window_end {
            self
        } else {
            Self::empty(window_end)
        }
    }
}

/// Agile history before `today` (London time) for `region`, from the cache when it already
/// covers that window.
///
/// Past prices never change, so entries stay valid until the window moves at London midnight,
/// when the whole cache is replaced.
pub async fn load_history(region: Region, today: NaiveDate) -> Result<Rates, AppError> {
    let (_, window_end) = history_window(today);

    if let Some(rates) = read(window_end).regions.remove(&region) {
        return Ok(rates);
    }

    let rates = fetch_agile_history(region, today).await?;
    // Read again: another region may have been cached while this one was being fetched.
    let mut cache = read(window_end);
    cache.regions.insert(region, rates.clone());
    write(cache, region);
    Ok(rates)
}

fn read(window_end: DateTime<Utc>) -> Cache {
    LocalStorage::get::<Cache>(KEY).map_or_else(
        |_| Cache::empty(window_end),
        |cache| cache.for_window(window_end),
    )
}

/// Saves `cache`, falling back to just `region` if every region doesn't fit in storage.
fn write(mut cache: Cache, region: Region) {
    if LocalStorage::set(KEY, &cache).is_ok() {
        return;
    }
    cache.regions.retain(|cached, _| *cached == region);
    if let Err(error) = LocalStorage::set(KEY, &cache) {
        log::warn(&format!("Failed to cache price history: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rates::Rate;
    use chrono::{Duration, TimeZone};

    fn window_end() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 11, 23, 0, 0).unwrap()
    }

    fn one_rate(price: f64) -> Rates {
        let valid_from = Utc.with_ymd_and_hms(2026, 4, 11, 12, 0, 0).unwrap();
        Rates::new(vec![Rate {
            value_inc_vat: price,
            value_exc_vat: price / 1.05,
            valid_from,
            valid_to: valid_from + Duration::minutes(30),
        }])
    }

    fn cache_with(regions: &[(Region, f64)]) -> Cache {
        let mut cache = Cache::empty(window_end());
        for &(region, price) in regions {
            cache.regions.insert(region, one_rate(price));
        }
        cache
    }

    #[test]
    fn regions_are_cached_side_by_side() {
        let cache = cache_with(&[(Region::C, 10.0), (Region::M, 20.0)]);

        let stored = serde_json::to_string(&cache).unwrap();
        let restored = serde_json::from_str::<Cache>(&stored)
            .unwrap()
            .for_window(window_end());

        assert_eq!(restored.regions.get(&Region::C), Some(&one_rate(10.0)));
        assert_eq!(restored.regions.get(&Region::M), Some(&one_rate(20.0)));
    }

    #[test]
    fn cache_for_another_window_is_dropped() {
        let next_end = window_end() + Duration::days(1);
        let moved = cache_with(&[(Region::C, 10.0)]).for_window(next_end);

        assert_eq!(moved, Cache::empty(next_end));
    }

    #[test]
    fn cache_from_an_older_format_is_dropped() {
        let old = Cache {
            version: VERSION - 1,
            ..cache_with(&[(Region::C, 10.0)])
        };

        assert_eq!(old.for_window(window_end()), Cache::empty(window_end()));
    }

    #[test]
    fn single_region_entry_from_the_previous_format_does_not_parse() {
        let previous =
            r#"{"version":2,"region":"C","window_end":"2026-04-11T23:00:00Z","rates":[]}"#;
        assert!(serde_json::from_str::<Cache>(previous).is_err());
    }
}
