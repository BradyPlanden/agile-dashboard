use chrono::{DateTime, Duration, Utc};
use std::rc::Rc;
use yew::prelude::*;

use crate::models::rates::Rates;
use crate::utils::time::{london_time, slot_start};

#[derive(Properties, PartialEq)]
pub struct CheapestPeriodProps {
    pub rates: Option<Rc<Rates>>,
    pub now: DateTime<Utc>,
}

/// Displays the cheapest electricity period in the next 3 hours.
#[function_component(CheapestPeriod)]
pub fn cheapest_period(props: &CheapestPeriodProps) -> Html {
    let Some(rates) = props.rates.as_ref() else {
        return html! {};
    };

    let now = props.now;
    let window_start = slot_start(now);
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
