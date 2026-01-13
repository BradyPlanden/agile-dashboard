use crate::models::carbon::CarbonIntensity;
use crate::services::carbon_api::fetch_carbon_intensity;
use gloo_timers::future::TimeoutFuture;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum CarbonDataState {
    Loading,
    Loaded(Rc<CarbonIntensity>),
    Error(String),
}

impl CarbonDataState {
    pub fn data(&self) -> Option<&Rc<CarbonIntensity>> {
        match self {
            CarbonDataState::Loaded(data) => Some(data),
            _ => None,
        }
    }
}

#[hook]
pub fn use_carbon_intensity() -> UseStateHandle<CarbonDataState> {
    let state = use_state(|| CarbonDataState::Loading);
    let trigger = use_state(|| 0u32); // Polling trigger

    {
        let state = state.clone();
        let trigger_value = *trigger;

        use_effect_with(trigger_value, move |_| {
            let state = state.clone();
            let trigger = trigger.clone();

            spawn_local(async move {
                // Fetch carbon intensity data
                match fetch_carbon_intensity().await {
                    Ok(carbon_data) => state.set(CarbonDataState::Loaded(Rc::new(carbon_data))),
                    Err(e) => state.set(CarbonDataState::Error(e.to_string())),
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
