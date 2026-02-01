use std::cell::Cell;
use std::rc::Rc;
use yew::prelude::*;

use crate::models::rates::Rates;
use crate::services::api::{Region, fetch_rates_for_region};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, PartialEq, Debug)]
pub enum DataState {
    Loading,
    Loaded(Rc<Rates>),
    Error(String),
}

impl DataState {
    /// Returns true if the state is loading
    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns the data if it is loaded
    pub const fn data(&self) -> Option<&Rc<Rates>> {
        match self {
            Self::Loaded(rates) => Some(rates),
            _ => None,
        }
    }
}

#[hook]
pub fn use_rates(region: Region) -> UseStateHandle<DataState> {
    let state = use_state(|| DataState::Loading);
    let trigger = use_state(|| 0u32); // Polling trigger

    {
        let state = state.clone();
        let trigger_value = *trigger;

        use_effect_with((trigger_value, region), move |(_, region)| {
            let state = state.clone();
            let trigger = trigger;
            let region = *region;
            let aborted = Rc::new(Cell::new(false));
            let aborted_check = aborted.clone();

            // Reset to loading when region changes
            state.set(DataState::Loading);

            spawn_local(async move {
                // Fetch data for the specified region
                match fetch_rates_for_region(region).await {
                    Ok(rates) if !aborted_check.get() => {
                        state.set(DataState::Loaded(Rc::new(rates)));
                    }
                    Err(e) if !aborted_check.get() => {
                        state.set(DataState::Error(e.to_string()));
                    }
                    _ => {} // Request was aborted, ignore result
                }

                // Schedule next poll if enabled
                if crate::config::Config::ENABLE_AUTO_REFRESH && !aborted_check.get() {
                    TimeoutFuture::new(crate::config::Config::POLLING_INTERVAL_MS).await;
                    if !aborted_check.get() {
                        trigger.set(*trigger + 1); // Trigger next fetch
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
