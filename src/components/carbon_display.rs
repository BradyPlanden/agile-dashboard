use std::cmp::Ordering;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use yew::prelude::*;

use crate::models::carbon::{CarbonIntensity, IntensityIndex};
use crate::utils::time::london_hhmm;

#[derive(Properties, PartialEq)]
pub struct CarbonDisplayProps {
    pub data: Rc<CarbonIntensity>,
}

/// Latest measured and current forecast grid carbon intensity, with the trend between them.
#[function_component(CarbonDisplay)]
pub fn carbon_display(props: &CarbonDisplayProps) -> Html {
    let data = &props.data;
    let change = data.intensity_change();
    let (trend_class, trend_icon, trend_text) = match change.cmp(&0) {
        Ordering::Greater => ("trend-up", "↑", "Rising"),
        Ordering::Less => ("trend-down", "↓", "Falling"),
        Ordering::Equal => ("trend-flat", "→", "Steady"),
    };
    let (latest_from, latest_to) = data.latest_period();
    let (next_from, next_to) = data.next_period();

    html! {
        <div class="carbon-grid">
            <IntensityCard
                title="Latest measured"
                value={data.latest_value()}
                index={data.latest_index()}
                period={format_period(latest_from, latest_to)}
            />
            <IntensityCard
                title="Forecast now"
                value={data.next_forecast()}
                index={data.next_index()}
                period={format_period(next_from, next_to)}
            />
            <div class="stat carbon-trend">
                <h3>{ "Trend" }</h3>
                <p class={classes!("trend", trend_class)}>
                    <span class="trend-icon" aria-hidden="true">{ trend_icon }</span>
                    <span class="sr-only">{ trend_text }</span>
                    <span class="trend-value">
                        if change == 0 {
                            { "No change" }
                        } else {
                            { format!("{change:+} gCO₂/kWh") }
                        }
                    </span>
                </p>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct IntensityCardProps {
    title: AttrValue,
    value: u32,
    index: IntensityIndex,
    period: AttrValue,
}

#[function_component(IntensityCard)]
fn intensity_card(props: &IntensityCardProps) -> Html {
    html! {
        <div class="stat carbon-item">
            <h3>{ &props.title }</h3>
            <p class="carbon-value">
                { props.value }
                <span class="carbon-unit">{ " gCO₂/kWh" }</span>
            </p>
            <p class={classes!("carbon-index-badge", props.index.css_class())}>
                { props.index.label() }
            </p>
            <p class="carbon-time">{ &props.period }</p>
        </div>
    }
}

fn format_period(from: DateTime<Utc>, to: DateTime<Utc>) -> AttrValue {
    format!("{} – {}", london_hhmm(from), london_hhmm(to)).into()
}

#[cfg(test)]
mod tests {
    use super::format_period;
    use chrono::{TimeZone, Utc};

    #[test]
    fn formats_carbon_periods_in_london_time_during_bst() {
        let from = Utc.with_ymd_and_hms(2026, 4, 12, 14, 0, 0).unwrap();
        let to = Utc.with_ymd_and_hms(2026, 4, 12, 14, 30, 0).unwrap();

        assert_eq!(format_period(from, to), "15:00 – 15:30");
    }
}
