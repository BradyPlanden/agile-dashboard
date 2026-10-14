use std::rc::Rc;

use chrono::{DateTime, Duration, Utc};
use yew::prelude::*;

use crate::config::CHEAPEST_WINDOW_HOURS;
use crate::models::price_band::PriceBand;
use crate::models::rates::Rates;
use crate::utils::time::{london_hhmm, slot_start, start_label};

#[derive(Properties, PartialEq)]
pub struct NowPanelProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

/// The current and next Agile price, and the cheapest upcoming block of a few hours.
#[function_component(NowPanel)]
pub fn now_panel(props: &NowPanelProps) -> Html {
    let NowPanelProps { rates, now } = props;
    let current = rates.rate_at(*now);
    let next = rates.next_rate(*now);
    let window = rates.cheapest_window(slot_start(*now), Duration::hours(CHEAPEST_WINDOW_HOURS));

    html! {
        <dl class="now-tiles">
            <div class={classes!(
                "stat",
                "stat-now",
                current.map(|rate| PriceBand::for_price(rate.value_inc_vat).css_class()),
            )}>
                <dt>{ "Current price" }</dt>
                if let Some(rate) = current {
                    <dd class="stat-value">
                        { format!("{:.2}p", rate.value_inc_vat) }
                        <span class="stat-unit">{ "/kWh" }</span>
                    </dd>
                    <dd class="stat-sub">{ format!("until {}", london_hhmm(rate.valid_to)) }</dd>
                } else {
                    <dd class="stat-value">{ "N/A" }</dd>
                }
            </div>
            <div class="stat">
                <dt>{ "Next price" }</dt>
                if let Some(rate) = next {
                    <dd class="stat-value">{ format!("{:.2}p", rate.value_inc_vat) }</dd>
                    <dd class="stat-sub">{ format!("from {}", london_hhmm(rate.valid_from)) }</dd>
                } else {
                    <dd class="stat-value">{ "N/A" }</dd>
                    <dd class="stat-sub">{ "Not published yet" }</dd>
                }
            </div>
            <div class="stat">
                <dt>{ format!("Cheapest {CHEAPEST_WINDOW_HOURS} hours") }</dt>
                if let Some(window) = window {
                    <dd class="stat-value">
                        { start_label(window.start, *now) }
                        { format!("–{}", london_hhmm(window.end)) }
                    </dd>
                    <dd class="stat-sub">{ format!("average {:.2}p", window.average) }</dd>
                } else {
                    <dd class="stat-value">{ "N/A" }</dd>
                    <dd class="stat-sub">{ "Not enough prices published" }</dd>
                }
            </div>
        </dl>
    }
}
