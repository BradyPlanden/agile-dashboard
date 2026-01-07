use crate::models::rates::DayStats;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DaySummaryProps {
    pub stats: DayStats,
    pub title: String,
    pub current_price: Option<f64>,
    pub next_price: Option<f64>,
    #[prop_or(false)]
    pub is_tomorrow: bool,
}

#[function_component(DaySummary)]
pub fn day_summary(props: &DaySummaryProps) -> Html {
    let card_class = if props.is_tomorrow {
        "day-summary-card tomorrow"
    } else {
        "day-summary-card"
    };

    html! {
        <div class={card_class}>
            <h2>{&props.title}</h2>
            <div class="summary-grid">
                <div class="summary-item">
                    <h3>{"Price Range"}</h3>
                    <p class="summary-value">{&props.stats.price_range}</p>
                </div>
                <div class="summary-item">
                    <h3>{"Average Price"}</h3>
                    <p class="summary-value">{format!("{:.2}p", props.stats.avg)}</p>
                </div>
                if let Some(current) = props.current_price {
                    <div class="summary-item">
                        <h3>{"Current Price"}</h3>
                        <p class="summary-value">{format!("{:.2}p", current)}</p>
                    </div>
                }
                if let Some(next) = props.next_price {
                    <div class="summary-item">
                        <h3>{"Next Price"}</h3>
                        <p class="summary-value">{format!("{:.2}p", next)}</p>
                    </div>
                }
            </div>
        </div>
    }
}
