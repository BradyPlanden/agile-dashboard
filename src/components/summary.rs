use std::rc::Rc;

use chrono::{DateTime, Utc};
use yew::prelude::*;

use crate::components::DaySummary;
use crate::models::rates::Rates;
use crate::utils::time::london_date;

#[derive(Properties, PartialEq)]
pub struct SummaryProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

/// Agile price statistics for today and tomorrow.
#[function_component(Summary)]
pub fn summary(props: &SummaryProps) -> Html {
    let today = london_date(props.now);
    let tomorrow = today.succ_opt().expect("tomorrow is a valid date");

    html! {
        <div class="day-cards">
            <DaySummary
                title="Today"
                stats={props.rates.stats_for_date(today)}
                placeholder="No prices published for today."
            />
            <DaySummary
                title="Tomorrow"
                stats={props.rates.stats_for_date(tomorrow)}
                placeholder="Not published yet. Octopus usually publishes tomorrow's prices around 4pm."
                highlight={true}
            />
        </div>
    }
}
