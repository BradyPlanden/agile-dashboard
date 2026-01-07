use crate::components::DaySummary;
use crate::models::rates::Rates;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SummaryProps {
    pub rates: Rc<Rates>,
}

#[function_component(Summary)]
pub fn summary(props: &SummaryProps) -> Html {
    let daily_stats = use_memo(props.rates.clone(), |rates| rates.daily_stats());

    match &*daily_stats {
        Ok(stats) => html! {
            <div class="data-summary">
                // Today's card (always shown)
                <DaySummary
                    stats={stats.today.clone()}
                    title={"Today's Statistics"}
                    current_price={Some(stats.current)}
                    next_price={Some(stats.next)}
                    is_tomorrow={false}
                />

                // Tomorrow's card (conditional)
                if let Some(tomorrow) = &stats.tomorrow {
                    <DaySummary
                        stats={tomorrow.clone()}
                        title={"Tomorrow's Statistics"}
                        current_price={None}
                        next_price={None}
                        is_tomorrow={true}
                    />
                }
            </div>
        },
        Err(e) => html! {
            <div class="data-summary error">
                <p>{"Error calculating summary: "}{e.to_string()}</p>
            </div>
        },
    }
}
