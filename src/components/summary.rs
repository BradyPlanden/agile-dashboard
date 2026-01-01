use crate::models::rates::Rates;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SummaryProps {
    pub rates: Rc<Rates>,
}

#[function_component(Summary)]
pub fn summary(props: &SummaryProps) -> Html {
    let stats_result = props.rates.stats();

    match stats_result {
        Ok(summary) => html! {
             <div class="data-summary">
                <div class="summary-grid">
                    <div class="summary-item">
                        <h3>{"Price Range"}</h3>
                        <p class="summary-value">{&summary.price_range}</p>
                    </div>
                    <div class="summary-item">
                        <h3>{"Average Price"}</h3>
                        <p class="summary-value">{format!("{:.2}p", summary.avg)}</p>
                    </div>
                    <div class="summary-item">
                        <h3>{"Current Price"}</h3>
                        <p class="summary-value">{format!("{:.2}p", summary.current)}</p>
                    </div>
                    <div class="summary-item">
                        <h3>{"Next Price"}</h3>
                        <p class="summary-value">{format!("{:.2}p", summary.next)}</p>
                    </div>
                </div>
            </div>
        },
        Err(e) => html! {
            <div class="data-summary error">
                <p>{"Error calculating summary: "}{e.to_string()}</p>
            </div>
        },
    }
}
