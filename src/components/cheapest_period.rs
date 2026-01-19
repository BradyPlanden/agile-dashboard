use chrono::{Duration, DurationRound, Local, Utc};
use yew::prelude::*;

use crate::hooks::use_rates::{DataState, use_rates};
use crate::hooks::use_region::use_region;

/// Displays the cheapest electricity period in the next 3 hours
#[function_component(CheapestPeriod)]
pub fn cheapest_period() -> Html {
    let region_handle = use_region();
    let state = use_rates(region_handle.region);

    let cheapest_time = match &*state {
        DataState::Loaded(rates) => {
            let now = Utc::now();
            let window_start = now
                .duration_trunc(Duration::minutes(30))
                .expect("30 minutes is a valid truncation duration");
            let three_hours_later = now + Duration::hours(3);

            // Find the cheapest rate in the next 3 hours (including current window)
            let cheapest = rates
                .filter_from(window_start) // Use window_start instead of now
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
