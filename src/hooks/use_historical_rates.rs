use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use chrono::{DateTime, Days, Duration, Utc};
use gloo_storage::Storage;
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::models::load_state::LoadState;
use crate::models::rates::Rates;
use crate::services::api::fetch_historical_rates_progressive;
use crate::utils::time::{london_date, london_midnight_utc};

const HISTORICAL_RATES_CACHE_KEY: &str = "historical_rates";
const HISTORICAL_RATES_CACHE_VERSION: u8 = 1;
const HISTORICAL_RATES_CACHE_MAX_AGE_HOURS: i64 = 24;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HistoricalRatesCache {
    version: u8,
    fetched_at: DateTime<Utc>,
    rates: Rates,
}

/// Hook that returns cached 31-day historical rates immediately, then refreshes in place.
#[hook]
pub fn use_historical_rates() -> LoadState<Rates> {
    let state = use_state(|| {
        load_cached_historical_rates().map_or(LoadState::Loading, |rates| {
            LoadState::Loaded(Rc::new(rates))
        })
    });
    let baseline_rates = match &*state {
        LoadState::Loaded(rates) => Some((**rates).clone()),
        _ => None,
    };

    {
        let state = state.clone();
        let baseline_rates = baseline_rates.clone();

        use_effect_with((), move |()| {
            let state = state.clone();
            let mut baseline_rates = baseline_rates.clone();
            let aborted = Rc::new(Cell::new(false));
            let aborted_check = aborted.clone();

            spawn_local(async move {
                loop {
                    let progress_base = baseline_rates
                        .clone()
                        .map(|rates| trim_to_current_historical_window(&rates, Utc::now()));
                    let state_for_progress = state.clone();
                    let aborted_for_progress = aborted_check.clone();
                    let latest_progress = Rc::new(RefCell::new(progress_base.clone()));
                    let latest_progress_for_callback = latest_progress.clone();

                    match fetch_historical_rates_progressive(move |fresh_rates| {
                        let display_rates = progress_base.as_ref().map_or_else(
                            || fresh_rates.clone(),
                            |base| base.merged_with(&fresh_rates),
                        );

                        latest_progress_for_callback.replace(Some(display_rates.clone()));

                        if !aborted_for_progress.get() {
                            state_for_progress.set(LoadState::Loaded(Rc::new(display_rates)));
                        }
                    })
                    .await
                    {
                        Ok(result) if !aborted_check.get() => {
                            if result.complete {
                                save_historical_rates(&result.rates);
                                baseline_rates = Some(result.rates.clone());
                                state.set(LoadState::Loaded(Rc::new(result.rates)));
                            } else if let Some(display_rates) = latest_progress.borrow().clone() {
                                baseline_rates = Some(display_rates.clone());
                                state.set(LoadState::Loaded(Rc::new(display_rates)));
                                warn("Historical refresh completed with partial data");
                            }
                        }
                        Err(error) if !aborted_check.get() => {
                            if matches!(&*state, LoadState::Loaded(_)) {
                                warn(&format!(
                                    "Historical refresh failed, continuing to use cached banner data: {error}"
                                ));
                            } else {
                                state.set(LoadState::Error(error.to_string()));
                            }
                        }
                        _ => {}
                    }

                    if !crate::config::Config::ENABLE_AUTO_REFRESH || aborted_check.get() {
                        break;
                    }

                    TimeoutFuture::new(crate::config::Config::POLLING_INTERVAL_MS).await;

                    if aborted_check.get() {
                        break;
                    }
                }
            });

            move || aborted.set(true)
        });
    }

    (*state).clone()
}

fn load_cached_historical_rates() -> Option<Rates> {
    let cache: HistoricalRatesCache =
        gloo_storage::LocalStorage::get(HISTORICAL_RATES_CACHE_KEY).ok()?;

    if cache.version != HISTORICAL_RATES_CACHE_VERSION {
        return None;
    }

    if !cache_is_fresh(cache.fetched_at, Utc::now()) {
        return None;
    }

    Some(trim_to_current_historical_window(&cache.rates, Utc::now()))
}

fn save_historical_rates(rates: &Rates) {
    let cache = HistoricalRatesCache {
        version: HISTORICAL_RATES_CACHE_VERSION,
        fetched_at: Utc::now(),
        rates: rates.clone(),
    };

    if let Err(error) = gloo_storage::LocalStorage::set(HISTORICAL_RATES_CACHE_KEY, cache) {
        warn(&format!("Failed to persist historical cache: {error:?}"));
    }
}

fn cache_is_fresh(fetched_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now.signed_duration_since(fetched_at) <= Duration::hours(HISTORICAL_RATES_CACHE_MAX_AGE_HOURS)
}

fn trim_to_current_historical_window(rates: &Rates, now: DateTime<Utc>) -> Rates {
    let today = london_date(now);
    let start_date = today
        .checked_sub_days(Days::new(31))
        .expect("historical start date should be valid");
    let start = london_midnight_utc(start_date);
    let end = london_midnight_utc(today);
    rates.within_window(start, end)
}

fn warn(message: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::warn_1(&message.into());

    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn cache_is_fresh_with_recent_snapshot() {
        let fetched_at = Utc.with_ymd_and_hms(2026, 4, 12, 10, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 4, 13, 9, 59, 59).unwrap();

        assert!(cache_is_fresh(fetched_at, now));
    }

    #[test]
    fn cache_is_stale_after_max_age() {
        let fetched_at = Utc.with_ymd_and_hms(2026, 4, 12, 10, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 1).unwrap();

        assert!(!cache_is_fresh(fetched_at, now));
    }

    #[test]
    fn trim_to_current_historical_window_drops_out_of_range_cached_slots() {
        let start = london_midnight_utc(chrono::NaiveDate::from_ymd_opt(2026, 3, 12).unwrap());
        let end = london_midnight_utc(chrono::NaiveDate::from_ymd_opt(2026, 4, 12).unwrap());

        let rates = Rates::new(vec![
            crate::models::rates::Rate {
                value_inc_vat: 10.0,
                value_exc_vat: 8.33,
                valid_from: start - Duration::minutes(30),
                valid_to: start,
            },
            crate::models::rates::Rate {
                value_inc_vat: 20.0,
                value_exc_vat: 16.67,
                valid_from: start,
                valid_to: start + Duration::minutes(30),
            },
            crate::models::rates::Rate {
                value_inc_vat: 30.0,
                value_exc_vat: 25.0,
                valid_from: end - Duration::minutes(30),
                valid_to: end,
            },
        ]);

        let trimmed = trim_to_current_historical_window(
            &rates,
            Utc.with_ymd_and_hms(2026, 4, 12, 12, 0, 0).unwrap(),
        );

        assert_eq!(trimmed.all_values(), vec![20.0, 30.0]);
    }
}
