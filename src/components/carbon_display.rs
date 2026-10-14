use crate::models::carbon::CarbonIntensity;
use crate::utils::time::london_time;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CarbonDisplayProps {
    pub data: Rc<CarbonIntensity>,
}

#[function_component(CarbonDisplay)]
pub fn carbon_display(props: &CarbonDisplayProps) -> Html {
    let data = &props.data;

    // Last actual values
    let latest_intensity = data.latest_value();
    let latest_source = "Actual";
    let latest_index = data.latest_index();
    let latest_index_class = format!("carbon-index-badge {}", latest_index.css_class());
    let (latest_from, latest_to) = data.latest_period();
    let latest_time_period = format_period(latest_from, latest_to);

    // Next period values
    let next_intensity = data.next_forecast();
    let next_index = data.next_index();
    let next_index_class = format!("carbon-index-badge {}", next_index.css_class());
    let (next_from, next_to) = data.next_period();
    let next_time_period = format_period(next_from, next_to);

    // Trend indicator
    let intensity_change = data.intensity_change();
    let change_class = match intensity_change.cmp(&0) {
        std::cmp::Ordering::Greater => "carbon-change-increasing",
        std::cmp::Ordering::Less => "carbon-change-decreasing",
        std::cmp::Ordering::Equal => "carbon-change-stable",
    };
    let change_icon = match intensity_change.cmp(&0) {
        std::cmp::Ordering::Greater => "↑",
        std::cmp::Ordering::Less => "↓",
        std::cmp::Ordering::Equal => "→",
    };
    let change_text = match intensity_change.cmp(&0) {
        std::cmp::Ordering::Greater => "Increasing",
        std::cmp::Ordering::Less => "Decreasing",
        std::cmp::Ordering::Equal => "Stable",
    };

    html! {
        <div class="carbon-display" role="region" aria-label="Carbon intensity information">
            <div class="carbon-grid">
                // Current period - prominent display
                <div
                    class="carbon-item carbon-item-current"
                    aria-label={format!(
                        "Most recent carbon intensity: {} grams CO2 per kilowatt hour, rated {}. {}",
                        latest_intensity,
                        latest_index.label(),
                        change_text
                    )}
                >
                    <h3>{"Most Recent"}</h3>
                    <p class="carbon-value">
                        {format!("{} ", latest_intensity)}
                        <span class="carbon-unit">{"gCO₂/kWh"}</span>
                    </p>
                    <div class={latest_index_class}>
                        {latest_index.label()}
                    </div>
                    <p class="carbon-time">{latest_time_period}</p>
                    <p class="carbon-source">{latest_source}</p>
                    <span class="sr-only">{change_text}</span>
                </div>

                // Next period - secondary display
                <div class="carbon-item carbon-item-next">
                    <h3>{"Current Forecast"}</h3>
                    <p class="carbon-value">
                        {format!("{} ", next_intensity)}
                        <span class="carbon-unit">{"gCO₂/kWh"}</span>
                    </p>
                    <div class={next_index_class}>
                        {next_index.label()}
                    </div>
                    <p class="carbon-time">{next_time_period}</p>
                    <p class="carbon-source">{"Forecast"}</p>
                </div>

                // Trend indicator
                <div class="carbon-item carbon-item-change">
                    <h3>{"Trend"}</h3>
                    <div class={format!("carbon-change {}", change_class)}>
                        <span class="carbon-change-icon">{change_icon}</span>
                        <span class="carbon-change-value">
                            {if intensity_change == 0 {
                                "No change".to_string()
                            } else {
                                format!("{intensity_change:+} gCO₂/kWh")
                            }}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn format_period(from: chrono::DateTime<chrono::Utc>, to: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "{} - {}",
        london_time(from).format("%H:%M"),
        london_time(to).format("%H:%M")
    )
}

#[cfg(test)]
mod tests {
    use super::format_period;
    use chrono::{TimeZone, Utc};

    #[test]
    fn formats_carbon_periods_in_london_time_during_bst() {
        let from = Utc.with_ymd_and_hms(2026, 4, 12, 14, 0, 0).unwrap();
        let to = Utc.with_ymd_and_hms(2026, 4, 12, 14, 30, 0).unwrap();

        assert_eq!(format_period(from, to), "15:00 - 15:30");
    }
}
