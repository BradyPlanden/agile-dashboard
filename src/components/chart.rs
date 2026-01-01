use charming::{
    Chart as CharmingChart,
    component::{Axis, Grid, Title, VisualMap, VisualMapPiece},
    element::{
        AxisLabel, AxisPointer, AxisPointerType, AxisType, LineStyle, LineStyleType, SplitLine,
        TextStyle, Tooltip, Trigger,
    },
    renderer::WasmRenderer,
    series::Bar,
};
use gloo::events::EventListener;
use std::rc::Rc;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::models::rates::Rates;

const CHART_ID: &str = "energy-chart";

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub rates: Rc<Rates>,
}

#[function_component(Chart)]
pub fn chart(props: &ChartProps) -> Html {
    let container_ref = use_node_ref();

    {
        let rates = props.rates.clone();
        let container_ref = container_ref.clone();

        use_effect_with((rates, container_ref), |(rates, container_ref)| {
            let listener = container_ref.cast::<HtmlElement>().map(|container| {
                render_chart(&container, rates);

                let rates = rates.clone();
                EventListener::new(&web_sys::window().unwrap(), "resize", move |_| {
                    render_chart(&container, &rates);
                })
            });

            move || drop(listener)
        });
    }

    html! {
        <div class="chart-container" ref={container_ref}>
            <div id={CHART_ID} />
        </div>
    }
}

fn render_chart(container: &HtmlElement, rates: &Rates) {
    let width = container.client_width() as u32;
    let height = container.client_height() as u32;

    if width == 0 || height == 0 {
        return;
    }

    match build_chart(rates) {
        Ok(chart) => {
            if let Err(e) = WasmRenderer::new(width, height).render(CHART_ID, &chart) {
                web_sys::console::error_1(&format!("Render error: {e:?}").into());
            }
        }
        Err(e) => web_sys::console::error_1(&format!("Chart error: {e}").into()),
    }
}

fn build_chart(rates: &Rates) -> Result<CharmingChart, crate::models::error::AppError> {
    let (x_data, y_data) = rates.series_data()?;

    Ok(CharmingChart::new()
        .title(
            Title::new()
                .text("Energy Prices")
                .left("center")
                .text_style(TextStyle::new().font_size(16).color("#1f2937")),
        )
        .tooltip(
            Tooltip::new()
                .trigger(Trigger::Axis)
                .axis_pointer(AxisPointer::new().type_(AxisPointerType::Shadow)),
        )
        .visual_map(VisualMap::new().show(false).pieces(vec![
            VisualMapPiece::new().lt(15).color("#10b981"),
            VisualMapPiece::new().gte(15).lt(25).color("#f59e0b"),
            VisualMapPiece::new().gte(25).color("#ef4444"),
        ]))
        .grid(
            Grid::new()
                .left("8%")
                .right("4%")
                .bottom("18%")
                .contain_label(true),
        )
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(x_data)
                .axis_label(AxisLabel::new().rotate(45).color("#6b7280").interval(5)),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("p/kWh")
                .axis_label(AxisLabel::new().color("#6b7280"))
                .split_line(
                    SplitLine::new().line_style(
                        LineStyle::new()
                            .color("#e5e7eb")
                            .type_(LineStyleType::Dashed),
                    ),
                ),
        )
        .series(Bar::new().data(y_data).bar_width("70%")))
}
