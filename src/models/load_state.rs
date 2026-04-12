use std::rc::Rc;
use yew::prelude::*;

/// Generic loading state for async data.
///
/// Replaces the duplicated per-type enums (`DataState`, `HistoricalDataState`,
/// `TrackerDataState`, `CarbonDataState`) with a single parameterized type.
#[derive(Clone, PartialEq, Debug)]
pub enum LoadState<T> {
    Loading,
    Loaded(Rc<T>),
    Error(String),
}

impl<T> LoadState<T> {
    /// Returns the loaded data, if present.
    pub const fn data(&self) -> Option<&Rc<T>> {
        match self {
            Self::Loaded(data) => Some(data),
            _ => None,
        }
    }
}

/// A reusable polling hook that encapsulates the fetch → Loaded/Error →
/// schedule-next-poll pattern shared by all four data hooks.
///
/// `fetch_fn` is called on every trigger. On success the state becomes
/// `LoadState::Loaded`, on error `LoadState::Error`. If auto-refresh is
/// enabled the hook re-triggers after the configured polling interval.
#[hook]
pub fn use_polling<T, F, Fut>(fetch_fn: F) -> UseStateHandle<LoadState<T>>
where
    T: Clone + PartialEq + 'static,
    F: Fn() -> Fut + 'static,
    Fut: std::future::Future<Output = Result<T, crate::models::error::AppError>> + 'static,
{
    use std::cell::Cell;
    use std::rc::Rc;

    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen_futures::spawn_local;

    let state: UseStateHandle<LoadState<T>> = use_state(|| LoadState::Loading);
    let trigger = use_state(|| 0u32);
    let trigger_value = *trigger;

    {
        let state = state.clone();

        use_effect_with(trigger_value, move |_| {
            let state = state.clone();
            let trigger = trigger;
            let aborted = Rc::new(Cell::new(false));
            let aborted_check = aborted.clone();

            spawn_local(async move {
                match fetch_fn().await {
                    Ok(data) if !aborted_check.get() => {
                        state.set(LoadState::Loaded(Rc::new(data)));
                    }
                    Err(e) if !aborted_check.get() => {
                        state.set(LoadState::Error(e.to_string()));
                    }
                    _ => {}
                }

                if crate::config::Config::ENABLE_AUTO_REFRESH && !aborted_check.get() {
                    TimeoutFuture::new(crate::config::Config::POLLING_INTERVAL_MS).await;
                    if !aborted_check.get() {
                        trigger.set(*trigger + 1);
                    }
                }
            });

            move || {
                aborted.set(true);
            }
        });
    }

    state
}

/// Variant that resets to Loading when a region (or other key) changes.
#[hook]
pub fn use_polling_with_key<T, F, Fut, K>(fetch_fn: F, key: K) -> UseStateHandle<LoadState<T>>
where
    T: Clone + PartialEq + 'static,
    K: PartialEq + 'static,
    F: Fn() -> Fut + 'static,
    Fut: std::future::Future<Output = Result<T, crate::models::error::AppError>> + 'static,
{
    use std::cell::Cell;
    use std::rc::Rc;

    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen_futures::spawn_local;

    let state: UseStateHandle<LoadState<T>> = use_state(|| LoadState::Loading);
    let trigger = use_state(|| 0u32);
    let trigger_value = *trigger;

    {
        let state = state.clone();

        use_effect_with((trigger_value, key), move |(_, _key)| {
            let state = state.clone();
            let trigger = trigger;
            let aborted = Rc::new(Cell::new(false));
            let aborted_check = aborted.clone();

            // Reset to loading when key changes
            state.set(LoadState::Loading);

            spawn_local(async move {
                match fetch_fn().await {
                    Ok(data) if !aborted_check.get() => {
                        state.set(LoadState::Loaded(Rc::new(data)));
                    }
                    Err(e) if !aborted_check.get() => {
                        state.set(LoadState::Error(e.to_string()));
                    }
                    _ => {}
                }

                if crate::config::Config::ENABLE_AUTO_REFRESH && !aborted_check.get() {
                    TimeoutFuture::new(crate::config::Config::POLLING_INTERVAL_MS).await;
                    if !aborted_check.get() {
                        trigger.set(*trigger + 1);
                    }
                }
            });

            move || {
                aborted.set(true);
            }
        });
    }

    state
}
