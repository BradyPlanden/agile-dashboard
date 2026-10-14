use std::cell::Cell;
use std::future::Future;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::config::ERROR_RETRY_BASE_MS;
use crate::hooks::use_page_visible::{is_page_visible, use_page_visible};
use crate::models::error::AppError;
use crate::models::load_state::LoadState;

/// A polled resource and a callback that refetches it immediately.
#[derive(Debug, PartialEq)]
pub struct Polled<T> {
    pub state: LoadState<T>,
    pub refresh: Callback<()>,
}

impl<T> Clone for Polled<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            refresh: self.refresh.clone(),
        }
    }
}

/// Fetches `fetch(key)` now and every `interval_ms` while the page is visible.
///
/// - Changing `key` shows `Loading` until the new key's data arrives, never the old key's.
/// - A failed refresh keeps the last good data (`Stale`) and retries with backoff.
/// - Refreshes pause while the tab is hidden and catch up when it is shown again.
#[hook]
pub fn use_polling<K, T, F, Fut>(key: K, interval_ms: u32, fetch: F) -> Polled<T>
where
    K: Clone + PartialEq + 'static,
    T: 'static,
    F: Fn(K) -> Fut + 'static,
    Fut: Future<Output = Result<T, AppError>> + 'static,
{
    let visible = use_page_visible();
    let state = use_state(|| (key.clone(), LoadState::<T>::Loading));
    let refreshes = use_state(|| 0_u32);
    let handled_refreshes = use_mut_ref(|| 0_u32);

    {
        let state = state.clone();
        use_effect_with(
            (key.clone(), visible, *refreshes),
            move |(key, visible, refreshes)| {
                let cancelled = Rc::new(Cell::new(false));
                let mut current = if state.0 == *key {
                    state.1.clone()
                } else {
                    LoadState::Loading
                };
                // Always fetch something to show, even in a background tab; only refreshes wait.
                if *visible || current.snapshot().is_none() {
                    let forced = handled_refreshes.replace(*refreshes) != *refreshes;
                    let mut delay = if forced {
                        0
                    } else {
                        initial_delay(&current, Utc::now(), interval_ms)
                    };
                    let key = key.clone();
                    let cancelled = cancelled.clone();

                    spawn_local(async move {
                        let mut failures = 0;
                        loop {
                            if delay > 0 {
                                TimeoutFuture::new(delay).await;
                            }
                            if cancelled.get() {
                                return;
                            }
                            let result = fetch(key.clone()).await;
                            if cancelled.get() {
                                return;
                            }
                            failures = if result.is_ok() { 0 } else { failures + 1 };
                            current = current.apply(result, Utc::now());
                            state.set((key.clone(), current.clone()));
                            if !is_page_visible() {
                                return; // Resumed by the effect when the page is shown.
                            }
                            delay = next_delay(failures, interval_ms);
                        }
                    });
                }
                move || cancelled.set(true)
            },
        );
    }

    let refresh = Callback::from(move |()| refreshes.set(refreshes.wrapping_add(1)));
    let shown = if state.0 == key {
        state.1.clone()
    } else {
        LoadState::Loading
    };
    Polled {
        state: shown,
        refresh,
    }
}

/// How long to wait before the first fetch: whatever remains of the interval for fresh
/// data, otherwise nothing.
fn initial_delay<T>(state: &LoadState<T>, now: DateTime<Utc>, interval_ms: u32) -> u32 {
    let LoadState::Loaded(snapshot) = state else {
        return 0;
    };
    let interval = i64::from(interval_ms);
    let age = (now - snapshot.fetched_at).num_milliseconds();
    u32::try_from((interval - age).clamp(0, interval)).unwrap_or(0)
}

/// The regular interval after a success; exponential backoff after failures.
fn next_delay(consecutive_failures: u32, interval_ms: u32) -> u32 {
    match consecutive_failures {
        0 => interval_ms,
        n => ERROR_RETRY_BASE_MS
            .saturating_mul(2_u32.saturating_pow(n - 1))
            .min(interval_ms),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::load_state::Snapshot;
    use chrono::{Duration, TimeZone};

    const INTERVAL: u32 = 600_000;

    fn loaded_at(fetched_at: DateTime<Utc>) -> LoadState<u32> {
        LoadState::Loaded(Snapshot {
            data: Rc::new(1),
            fetched_at,
        })
    }

    #[test]
    fn fresh_data_waits_out_the_rest_of_the_interval() {
        let fetched = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let now = fetched + Duration::minutes(4);

        assert_eq!(initial_delay(&loaded_at(fetched), now, INTERVAL), 360_000);
    }

    #[test]
    fn overdue_missing_or_failed_data_fetches_immediately() {
        let fetched = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let now = fetched + Duration::minutes(30);

        assert_eq!(initial_delay(&loaded_at(fetched), now, INTERVAL), 0);
        assert_eq!(initial_delay(&LoadState::<u32>::Loading, now, INTERVAL), 0);
        assert_eq!(
            initial_delay(&LoadState::<u32>::Failed(AppError::Timeout), now, INTERVAL),
            0
        );
    }

    #[test]
    fn retries_back_off_up_to_the_interval() {
        assert_eq!(next_delay(0, INTERVAL), INTERVAL);
        assert_eq!(next_delay(1, INTERVAL), 30_000);
        assert_eq!(next_delay(2, INTERVAL), 60_000);
        assert_eq!(next_delay(3, INTERVAL), 120_000);
        assert_eq!(next_delay(10, INTERVAL), INTERVAL);
        assert_eq!(next_delay(u32::MAX, INTERVAL), INTERVAL);
    }
}
