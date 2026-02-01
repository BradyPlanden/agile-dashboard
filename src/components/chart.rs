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
    pub dark_mode: bool,
}

#[function_component(Chart)]
pub fn chart(props: &ChartProps) -> Html {
    let container_ref = use_node_ref();
    let series_data = use_memo(props.rates.clone(), |rates| rates.series_data());

    {
        let container_ref = container_ref.clone();
        let dark_mode = props.dark_mode;

        use_effect_with(
            (series_data, container_ref, dark_mode),
            |(series_data, container_ref, dark_mode)| {
                let listener = container_ref.cast::<HtmlElement>().map(|container| {
                    render_chart(&container, series_data, *dark_mode);

                    let series_data = series_data.clone();
                    let dark_mode = *dark_mode;
                    EventListener::new(&web_sys::window().unwrap(), "resize", move |_| {
                        render_chart(&container, &series_data, dark_mode);
                    })
                });

                move || drop(listener)
            },
        );
    }

    html! {
        <div class="chart-container" ref={container_ref}>
            <div id={CHART_ID} />
        </div>
    }
}

fn render_chart(
    container: &HtmlElement,
    series_data: &Result<(Vec<String>, Vec<f64>), crate::models::error::AppError>,
    dark_mode: bool,
) {
    let width = container.client_width().cast_unsigned();
    let height = container.client_height().cast_unsigned();

    if width == 0 || height == 0 {
        return;
    }

    match series_data {
        Ok(data) => {
            let chart = build_chart(data, dark_mode);
            if let Err(e) = WasmRenderer::new(width, height).render(CHART_ID, &chart) {
                web_sys::console::error_1(&format!("Render error: {e:?}").into());
            }
        }
        Err(e) => web_sys::console::error_1(&format!("Series data error: {e}").into()),
    }
}

fn build_chart(series_data: &(Vec<String>, Vec<f64>), dark_mode: bool) -> CharmingChart {
    let (x_data, y_data) = series_data;

    // Theme-aware colors
    let (title_color, axis_color, grid_color) = if dark_mode {
        ("#e4e4e7", "#a1a1aa", "#404040")
    } else {
        ("#1f2937", "#6b7280", "#e5e7eb")
    };

    // Bar colors - slightly brighter for dark mode
    let bar_colors = if dark_mode {
        vec![
            "#22d3b3", // brighter teal
            "#7ba3ff", // brighter blue
            "#9b7ef5", // brighter purple
            "#ff4d9f", // brighter magenta
            "#ff8033", // brighter orange
            "#ffc733", // brighter yellow
        ]
    } else {
        vec![
            "#00b4a0", // original teal
            "#648fff", // original blue
            "#785ef0", // original purple
            "#dc267f", // original magenta
            "#fe6100", // original orange
            "#ffb000", // original yellow
        ]
    };

    CharmingChart::new()
        .title(
            Title::new()
                .text("Energy Prices")
                .left("center")
                .text_style(TextStyle::new().font_size(16).color(title_color)),
        )
        .tooltip(
            Tooltip::new()
                .trigger(Trigger::Axis)
                .axis_pointer(AxisPointer::new().type_(AxisPointerType::Shadow)),
        )
        .visual_map(VisualMap::new().show(false).pieces(vec![
            VisualMapPiece::new().lt(7.5).color(bar_colors[0]),
            VisualMapPiece::new().gte(7.5).lt(11.25).color(bar_colors[1]),
            VisualMapPiece::new().gte(11.25).lt(15.0).color(bar_colors[2]),
            VisualMapPiece::new().gte(15.0).lt(22.5).color(bar_colors[3]),
            VisualMapPiece::new().gte(22.5).lt(30.0).color(bar_colors[4]),
            VisualMapPiece::new().gte(30.0).color(bar_colors[5]),
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
                .data(x_data.clone())
                .axis_label(AxisLabel::new().rotate(45).color(axis_color).interval(5)),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("p/kWh")
                .axis_label(AxisLabel::new().color(axis_color))
                .split_line(
                    SplitLine::new().line_style(
                        LineStyle::new()
                            .color(grid_color)
                            .type_(LineStyleType::Dashed),
                    ),
                ),
        )
        .series(Bar::new().data(y_data.clone()).bar_width("70%"))
}
