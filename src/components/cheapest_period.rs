use std::rc::Rc;

use chrono::{DateTime, Duration, Utc};
use yew::prelude::*;

use crate::config::CHEAPEST_SLOT_LOOKAHEAD_HOURS;
use crate::models::rates::Rates;
use crate::utils::time::{slot_start, start_label};

#[derive(Properties, PartialEq)]
pub struct CheapestPeriodProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

/// Header chip showing when the cheapest half hour in the next few hours starts.
#[function_component(CheapestPeriod)]
pub fn cheapest_period(props: &CheapestPeriodProps) -> Html {
    let lookahead = Duration::hours(CHEAPEST_SLOT_LOOKAHEAD_HOURS);
    let Some(rate) = props
        .rates
        .cheapest_between(slot_start(props.now), props.now + lookahead)
    else {
        return html! {};
    };

    let when = start_label(rate.valid_from, props.now);
    let description = format!(
        "Cheapest half hour in the next {CHEAPEST_SLOT_LOOKAHEAD_HOURS} hours: {when}, {:.2}p",
        rate.value_inc_vat
    );

    html! {
        <p class="chip" title={description.clone()}>
            <span aria-hidden="true">{ format!("\u{2615} {when}") }</span>
            <span class="sr-only">{ description }</span>
        </p>
    }
}
