use crate::models::load_state::LoadState;
use crate::models::rates::Rates;
use crate::services::api::fetch_historical_rates;
use yew::prelude::*;

/// Hook that fetches and polls 31-day historical Agile rates.
#[hook]
pub fn use_historical_rates() -> UseStateHandle<LoadState<Rates>> {
    crate::models::load_state::use_polling(fetch_historical_rates)
}
