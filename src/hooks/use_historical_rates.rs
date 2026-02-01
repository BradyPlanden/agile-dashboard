use std::cell::Cell;
use std::rc::Rc;
use yew::prelude::*;

use crate::models::rates::Rates;
use crate::services::api::fetch_historical_rates;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, PartialEq, Debug)]
pub enum HistoricalDataState {
    Loading,
    Loaded(Rc<Rates>),
    Error(String),
}

impl HistoricalDataState {
    /// Returns the data if it is loaded
    pub const fn data(&self) -> Option<&Rc<Rates>> {
        match self {
            Self::Loaded(rates) => Some(rates),
            _ => None,
        }
    }
}

#[hook]
pub fn use_historical_rates() -> UseStateHandle<HistoricalDataState> {
    let state = use_state(|| HistoricalDataState::Loading);
    let trigger = use_state(|| 0u32); // Polling trigger

    {
        let state = state.clone();
        let trigger_value = *trigger;

        use_effect_with(trigger_value, move |_| {
            let state = state.clone();
            let trigger = trigger;
            let aborted = Rc::new(Cell::new(false));
            let aborted_check = aborted.clone();

            spawn_local(async move {
                // Fetch historical data
                match fetch_historical_rates().await {
                    Ok(rates) if !aborted_check.get() => {
                        state.set(HistoricalDataState::Loaded(Rc::new(rates)));
                    }
                    Err(e) if !aborted_check.get() => {
                        state.set(HistoricalDataState::Error(e.to_string()));
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
