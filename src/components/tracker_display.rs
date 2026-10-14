use std::rc::Rc;

use chrono::{DateTime, Utc};
use yew::prelude::*;

use crate::models::rates::Rates;
use crate::utils::time::london_date;

#[derive(Properties, PartialEq)]
pub struct TrackerDisplayProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

/// Tracker's daily price today and tomorrow.
#[function_component(TrackerDisplay)]
pub fn tracker_display(props: &TrackerDisplayProps) -> Html {
    let tomorrow = london_date(props.now)
        .succ_opt()
        .expect("tomorrow is a valid date");
    let today_price = props
        .rates
        .rate_at(props.now)
        .map(|rate| rate.value_inc_vat);
    let tomorrow_price = props
        .rates
        .for_date(tomorrow)
        .first()
        .map(|rate| rate.value_inc_vat);

    html! {
        <dl class="stat-grid">
            <div class="stat">
                <dt>{ "Today" }</dt>
                <dd class="stat-value">
                    { today_price.map_or_else(|| "N/A".to_owned(), |price| format!("{price:.2}p/kWh")) }
                </dd>
            </div>
            <div class="stat stat-highlight">
                <dt>{ "Tomorrow" }</dt>
                <dd class="stat-value">
                    if let Some(price) = tomorrow_price {
                        { format!("{price:.2}p/kWh ") }
                        if let Some(today) = today_price {
                            <span class={classes!("delta", if price > today { "delta-up" } else { "delta-down" })}>
                                { format!("({:+.2}p)", price - today) }
                            </span>
                        }
                    } else {
                        <span class="placeholder">{ "Not published yet" }</span>
                    }
                </dd>
            </div>
        </dl>
    }
}
