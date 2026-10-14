use std::rc::Rc;

use chrono::{DateTime, Utc};
use yew::prelude::*;

use crate::components::CheapestWindow;
use crate::models::price_band::PriceBand;
use crate::models::rates::Rates;
use crate::utils::time::london_hhmm;

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
            <CheapestWindow rates={rates.clone()} now={*now} />
        </dl>
    }
}
