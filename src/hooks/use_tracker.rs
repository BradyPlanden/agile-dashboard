use crate::models::rates::TrackerRates;
use crate::services::api::{Region, fetch_tracker_rates_for_region};
use gloo_timers::future::TimeoutFuture;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum TrackerDataState {
    Loading,
    Loaded(Rc<TrackerRates>),
    Error(String),
}

impl TrackerDataState {
    pub const fn data(&self) -> Option<&Rc<TrackerRates>> {
        match self {
            Self::Loaded(rates) => Some(rates),
            _ => None,
        }
    }
}

#[hook]
pub fn use_tracker_rates(region: Region) -> UseStateHandle<TrackerDataState> {
    let state = use_state(|| TrackerDataState::Loading);
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
            state.set(TrackerDataState::Loading);

            spawn_local(async move {
                // Fetch data for the specified region
                match fetch_tracker_rates_for_region(region).await {
                    Ok(rates) if !aborted_check.get() => {
                        state.set(TrackerDataState::Loaded(Rc::new(rates)));
                    }
                    Err(e) if !aborted_check.get() => {
                        state.set(TrackerDataState::Error(e.to_string()));
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
