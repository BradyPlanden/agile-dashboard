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

fn visible_state_for_key<T, K>(state: &LoadState<T>, state_key: &K, active_key: &K) -> LoadState<T>
where
    T: Clone,
    K: PartialEq,
{
    if state_key == active_key {
        state.clone()
    } else {
        LoadState::Loading
    }
}

/// A reusable polling hook that encapsulates the fetch → Loaded/Error →
/// schedule-next-poll pattern shared by all four data hooks.
///
/// `fetch_fn` is called on every trigger. On success the state becomes
/// `LoadState::Loaded`, on error `LoadState::Error`. If auto-refresh is
/// enabled the hook re-triggers after the configured polling interval.
#[hook]
pub fn use_polling<T, F, Fut>(fetch_fn: F) -> LoadState<T>
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

    (*state).clone()
}

/// Variant that resets to Loading when a region (or other key) changes.
#[hook]
pub fn use_polling_with_key<T, F, Fut, K>(fetch_fn: F, key: K) -> LoadState<T>
where
    T: Clone + PartialEq + 'static,
    K: Clone + PartialEq + 'static,
    F: Fn() -> Fut + 'static,
    Fut: std::future::Future<Output = Result<T, crate::models::error::AppError>> + 'static,
{
    use std::cell::Cell;
    use std::rc::Rc;

    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen_futures::spawn_local;

    let state: UseStateHandle<LoadState<T>> = use_state(|| LoadState::Loading);
    let initial_state_key = key.clone();
    let state_key = use_state(move || initial_state_key);
    let trigger = use_state(|| 0u32);
    let trigger_value = *trigger;

    {
        let state = state.clone();
        let state_key = state_key.clone();
        let effect_key = key.clone();

        use_effect_with((trigger_value, key.clone()), move |(_, _key)| {
            let state = state.clone();
            let state_key = state_key.clone();
            let trigger = trigger;
            let aborted = Rc::new(Cell::new(false));
            let aborted_check = aborted.clone();
            let request_key = effect_key.clone();

            spawn_local(async move {
                match fetch_fn().await {
                    Ok(data) if !aborted_check.get() => {
                        state_key.set(request_key.clone());
                        state.set(LoadState::Loaded(Rc::new(data)));
                    }
                    Err(e) if !aborted_check.get() => {
                        state_key.set(request_key);
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

    visible_state_for_key(&state, &*state_key, &key)
}

#[cfg(test)]
mod tests {
    use super::LoadState;
    use super::visible_state_for_key;
    use std::rc::Rc;

    #[test]
    fn visible_state_for_key_keeps_matching_loaded_data() {
        let state = LoadState::Loaded(Rc::new(42));

        let visible = visible_state_for_key(&state, &"region-a", &"region-a");

        assert_eq!(visible, state);
    }

    #[test]
    fn visible_state_for_key_hides_loaded_data_for_previous_key() {
        let state = LoadState::Loaded(Rc::new(42));

        let visible = visible_state_for_key(&state, &"region-a", &"region-b");

        assert_eq!(visible, LoadState::Loading);
    }
}
