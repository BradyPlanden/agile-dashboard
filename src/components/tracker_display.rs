use crate::models::rates::Rates;
use crate::utils::time::london_date;
use chrono::{DateTime, Utc};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TrackerDisplayProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

#[function_component(TrackerDisplay)]
pub fn tracker_display(props: &TrackerDisplayProps) -> Html {
    let now = props.now;
    let tomorrow = london_date(now)
        .succ_opt()
        .expect("tomorrow is a valid date");
    let current = props.rates.rate_at(now).map(|rate| rate.value_inc_vat);
    let next_day = props
        .rates
        .for_date(tomorrow)
        .first()
        .map(|rate| rate.value_inc_vat);

    html! {
        <div class="tracker-display">
            <div class="tracker-grid">
                <div class="tracker-item">
                    <h3>{"Current Price"}</h3>
                    <p class="tracker-value">
                        {
                            if let Some(price) = current {
                                format!("{price:.2}p/kWh")
                            } else {
                                "N/A".to_string()
                            }
                        }
                    </p>
                </div>
                <div class="tracker-item-tomorrow">
                    <h3>{"Tomorrow's Price"}</h3>
                    <p class="tracker-value">
                        {
                            match (next_day, current) {
                                (Some(price), Some(today)) => {
                                    let difference = price - today;
                                    let class = if difference >= 0.0 { "price-increase" } else { "price-decrease" };
                                    html! {
                                        <>
                                            {format!("{price:.2}p/kWh ")}
                                            <span class={class}>{format!("({difference:+.2}p)")}</span>
                                        </>
                                    }
                                },
                                (Some(price), None) => html! { {format!("{price:.2}p/kWh")} },
                                (None, _) => html! { {"Awaiting data"} },
                            }
                        }
                    </p>
                </div>
            </div>
        </div>
    }
}
