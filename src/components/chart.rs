use crate::models::rates::Rates;
use yew::prelude::*;
use yew_plotly::Plotly;
use yew_plotly::plotly::{Bar, Layout, Plot};

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub rates: Rates,
}

#[function_component(Chart)]
pub fn chart(props: &ChartProps) -> Html {
    let plot = create_plotly_chart(&props.rates);

    match plot {
        Ok(plot) => html! {
            <div class="chart-container">
                <Plotly plot={plot} />
            </div>
        },
        Err(e) => html! {
            <div class="chart-error">
                <p>{"Unable to render chart: "}{e.to_string()}</p>
            </div>
        },
    }
}

/// Plotly chart from current rates
fn create_plotly_chart(rates: &Rates) -> Result<Plot, crate::models::error::AppError> {
    let (x_data, y_data) = rates.series_data()?;

    // Bar chart
    let trace = Bar::new(x_data, y_data).name("Energy Prices");

    // Layout
    let layout = Layout::new()
        .title("Energy Price Distribution".into())
        .x_axis(
            yew_plotly::plotly::layout::Axis::new()
                .title("Time".into())
                .type_(yew_plotly::plotly::layout::AxisType::Date)
                .tick_format("%H:%M")
                .tick_angle(-45.0),
        )
        .y_axis(yew_plotly::plotly::layout::Axis::new().title("Price (p/kWh)".into()))
        .height(500)
        .margin(yew_plotly::plotly::layout::Margin::new().bottom(100));

    // Create plot
    let mut plot = Plot::new();
    plot.add_trace(trace);
    plot.set_layout(layout);

    Ok(plot)
}
