use crate::models::rates::TrackerRates;
use crate::services::api::fetch_tracker_rates;
use gloo_timers::future::TimeoutFuture;
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
    pub fn data(&self) -> Option<&Rc<TrackerRates>> {
        match self {
            TrackerDataState::Loaded(rates) => Some(rates),
            _ => None,
        }
    }
}

#[hook]
pub fn use_tracker_rates() -> UseStateHandle<TrackerDataState> {
    let state = use_state(|| TrackerDataState::Loading);
    let trigger = use_state(|| 0u32); // Polling trigger

    {
        let state = state.clone();
        let trigger_value = *trigger;

        use_effect_with(trigger_value, move |_| {
            let state = state.clone();
            let trigger = trigger.clone();

            spawn_local(async move {
                // Fetch data
                match fetch_tracker_rates().await {
                    Ok(rates) => state.set(TrackerDataState::Loaded(Rc::new(rates))),
                    Err(e) => state.set(TrackerDataState::Error(e.to_string())),
                }

                // Schedule next poll if enabled
                if crate::config::Config::ENABLE_AUTO_REFRESH {
                    TimeoutFuture::new(crate::config::Config::POLLING_INTERVAL_MS).await;
                    trigger.set(*trigger + 1); // Trigger next fetch
                }
            });

            || () // Cleanup
        });
    }

    state
}
