use crate::components::DaySummary;
use crate::models::rates::Rates;
use crate::utils::time::london_date;
use chrono::{DateTime, Utc};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SummaryProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

#[function_component(Summary)]
pub fn summary(props: &SummaryProps) -> Html {
    let now = props.now;
    let today = london_date(now);
    let tomorrow = today.succ_opt().expect("tomorrow is a valid date");

    match props.rates.stats_for_date(today) {
        Some(today_stats) => html! {
            <div class="data-summary">
                // Today's card (always shown)
                <DaySummary
                    stats={today_stats}
                    title={"Today's Statistics"}
                    current_price={props.rates.rate_at(now).map(|rate| rate.value_inc_vat)}
                    next_price={props.rates.next_rate(now).map(|rate| rate.value_inc_vat)}
                    is_tomorrow={false}
                />

                // Tomorrow's card (conditional)
                if let Some(tomorrow_stats) = props.rates.stats_for_date(tomorrow) {
                    <DaySummary
                        stats={tomorrow_stats}
                        title={"Tomorrow's Statistics"}
                        current_price={None}
                        next_price={None}
                        is_tomorrow={true}
                    />
                }
            </div>
        },
        None => html! {
            <div class="data-summary error">
                <p>{"No prices published for today"}</p>
            </div>
        },
    }
}
