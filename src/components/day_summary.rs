use yew::prelude::*;

use crate::models::rates::PriceStats;

#[derive(Properties, PartialEq)]
pub struct DaySummaryProps {
    pub title: AttrValue,
    pub stats: Option<PriceStats>,
    /// Shown instead of the statistics when there are none.
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub highlight: bool,
}

/// Low, average and high price for one day.
#[function_component(DaySummary)]
pub fn day_summary(props: &DaySummaryProps) -> Html {
    html! {
        <div class={classes!("day-card", props.highlight.then_some("highlight"))}>
            <h3>{ &props.title }</h3>
            if let Some(stats) = props.stats {
                <dl class="stat-grid">
                    <div class="stat">
                        <dt>{ "Low" }</dt>
                        <dd class="stat-value">{ format!("{:.2}p", stats.min) }</dd>
                    </div>
                    <div class="stat">
                        <dt>{ "Average" }</dt>
                        <dd class="stat-value">{ format!("{:.2}p", stats.avg) }</dd>
                    </div>
                    <div class="stat">
                        <dt>{ "High" }</dt>
                        <dd class="stat-value">{ format!("{:.2}p", stats.max) }</dd>
                    </div>
                </dl>
            } else {
                <p class="placeholder">{ &props.placeholder }</p>
            }
        </div>
    }
}
