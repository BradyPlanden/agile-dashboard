use chrono::{Duration, DurationRound, Utc};
use std::rc::Rc;
use yew::prelude::*;

use crate::models::rates::Rates;
use crate::utils::time::london_time;

#[derive(Properties, PartialEq)]
pub struct CheapestPeriodProps {
    pub rates: Option<Rc<Rates>>,
}

/// Displays the cheapest electricity period in the next 3 hours.
#[function_component(CheapestPeriod)]
pub fn cheapest_period(props: &CheapestPeriodProps) -> Html {
    let Some(rates) = props.rates.as_ref() else {
        return html! {};
    };

    let now = Utc::now();
    let window_start = now
        .duration_trunc(Duration::minutes(30))
        .expect("30 minutes is a valid truncation duration");
    let three_hours_later = now + Duration::hours(3);

    let cheapest_time = rates
        .cheapest_between(window_start, three_hours_later)
        .map(|rate| london_time(rate.valid_from).format("%H:%M").to_string());

    match cheapest_time {
        Some(time) => html! {
            <div class="cheapest-period" title="Cheapest period in next 3 hours">
                {"\u{2615} "}{time}
            </div>
        },
        None => html! {},
    }
}
