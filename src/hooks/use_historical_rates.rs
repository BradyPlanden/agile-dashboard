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

            spawn_local(async move {
                // Fetch historical data
                match fetch_historical_rates().await {
                    Ok(rates) => state.set(HistoricalDataState::Loaded(Rc::new(rates))),
                    Err(e) => state.set(HistoricalDataState::Error(e.to_string())),
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
