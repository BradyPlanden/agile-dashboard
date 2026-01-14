use chrono::{Duration, Local, Utc};
use yew::prelude::*;

use crate::hooks::use_rates::{DataState, use_rates};

/// Displays the cheapest electricity period in the next 3 hours
#[function_component(CheapestPeriod)]
pub fn cheapest_period() -> Html {
    let state = use_rates();

    let cheapest_time = match &*state {
        DataState::Loaded(rates) => {
            let now = Utc::now();
            let three_hours_later = now + Duration::hours(3);

            // Find the cheapest rate in the next 3 hours
            let cheapest = rates
                .filter_from(now)
                .take_while(|r| r.valid_from < three_hours_later)
                .min_by(|a, b| {
                    a.value_inc_vat
                        .partial_cmp(&b.value_inc_vat)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            cheapest.map(|rate| {
                // Convert to local time and format as HH:MM
                let local_time = rate.valid_from.with_timezone(&Local);
                local_time.format("%H:%M").to_string()
            })
        }
        _ => None,
    };

    match cheapest_time {
        Some(time) => html! {
            <div class="cheapest-period" title="Cheapest period in next 3 hours">
                {"\u{2615} "}{time}
            </div>
        },
        None => html! {},
    }
}
