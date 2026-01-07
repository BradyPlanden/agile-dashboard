use crate::models::rates::TrackerRates;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TrackerDisplayProps {
    pub rates: Rc<TrackerRates>,
}

#[function_component(TrackerDisplay)]
pub fn tracker_display(props: &TrackerDisplayProps) -> Html {
    // Single memoized computation for all three values
    let prices = use_memo(props.rates.clone(), |rates| {
        (
            rates.current_price(),
            rates.next_day_price(),
            rates.price_difference(),
        )
    });

    let (current, next_day, diff) = &*prices;

    html! {
        <div class="tracker-display">
            <div class="tracker-grid">
                <div class="tracker-item">
                    <h3>{"Current Price"}</h3>
                    <p class="tracker-value">
                        {
                            if let Some(price) = current {
                                format!("{:.2}p/kWh", price)
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
                            match (next_day, diff) {
                                (Some(price), Some(difference)) => {
                                    let sign = if *difference >= 0.0 { "+" } else { "" };
                                    let class = if *difference >= 0.0 { "price-increase" } else { "price-decrease" };
                                    html! {
                                        <>
                                            {format!("{:.2}p/kWh ", price)}
                                            <span class={class}>
                                                {format!("({}{}p)", sign, format!("{:.2}", difference))}
                                            </span>
                                        </>
                                    }
                                },
                                (Some(price), None) => html! { {format!("{:.2}p/kWh", price)} },
                                (None, _) => html! { {"Awaiting data"} },
                            }
                        }
                    </p>
                </div>
            </div>
        </div>
    }
}
