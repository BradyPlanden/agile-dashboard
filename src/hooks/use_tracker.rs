use crate::models::rates::TrackerRates;
use crate::services::api::fetch_tracker_rates;
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

    {
        let state = state.clone();
        use_effect(move || {
            spawn_local(async move {
                match fetch_tracker_rates().await {
                    Ok(rates) => state.set(TrackerDataState::Loaded(Rc::new(rates))),
                    Err(e) => state.set(TrackerDataState::Error(e.to_string())),
                }
            });
            || ()
        });
    }

    state
}
