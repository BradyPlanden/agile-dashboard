use crate::models::carbon::CarbonIntensity;
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
    let latest_intensity = data.latest_intensity();
    let latest_source = if data.has_actual() {
        "Actual"
    } else {
        "Actual Unavailable"
    };
    let latest_index = data.latest_index();
    let latest_index_class = format!("carbon-index-badge {}", latest_index.css_class());
    let (latest_from, latest_to) = data.latest_period();
    let latest_time_period = format!(
        "{} - {}",
        latest_from.format("%H:%M"),
        latest_to.format("%H:%M")
    );

    // Next period values
    let next_intensity = data.next_intensity();
    let next_index = data.next_index();
    let next_index_class = format!("carbon-index-badge {}", next_index.css_class());
    let (next_from, next_to) = data.next_period();
    let next_time_period = format!(
        "{} - {}",
        next_from.format("%H:%M"),
        next_to.format("%H:%M")
    );

    // Trend indicator
    let intensity_change = data.intensity_change();
    let change_class = if intensity_change > 0 {
        "carbon-change-increasing"
    } else if intensity_change < 0 {
        "carbon-change-decreasing"
    } else {
        "carbon-change-stable"
    };
    let change_icon = if intensity_change > 0 {
        "↑"
    } else if intensity_change < 0 {
        "↓"
    } else {
        "→"
    };

    html! {
        <div class="carbon-display">
            <div class="carbon-grid">
                // Current period - prominent display
                <div class="carbon-item carbon-item-current">
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
                                format!("{:+} gCO₂/kWh", intensity_change)
                            }}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}
