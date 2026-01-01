use std::rc::Rc;
use yew::prelude::*;

use crate::models::rates::Rates;
use crate::services::api::fetch_rates;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, PartialEq, Debug)]
pub enum DataState {
    Loading,
    Loaded(Rc<Rates>),
    Error(String),
}

impl DataState {
    /// Returns true if the state is loading
    pub fn is_loading(&self) -> bool {
        matches!(self, DataState::Loading)
    }

    /// Returns the data if it is loaded
    pub fn data(&self) -> Option<&Rc<Rates>> {
        match self {
            DataState::Loaded(rates) => Some(rates),
            _ => None,
        }
    }
}

#[hook]
pub fn use_rates() -> UseStateHandle<DataState> {
    let state = use_state(|| DataState::Loading);

    {
        let state = state.clone();
        use_effect(move || {
            spawn_local(async move {
                match fetch_rates().await {
                    Ok(rates) => state.set(DataState::Loaded(Rc::new(rates))),
                    Err(e) => state.set(DataState::Error(e.to_string())),
                }
            });
            || ()
        });
    }

    state
}
