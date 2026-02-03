use crate::utils::debounce::create_debounced_resize_listener;
use web_sys::HtmlElement;
use yew::prelude::*;

/// Generates SVG path data from values
#[allow(clippy::cast_precision_loss)]
pub fn build_path(values: &[f64], width: f64, height: f64, padding: f64) -> String {
    if values.is_empty() {
        return String::new();
    }

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < 0.01 {
        1.0 // Avoid division by zero for flat lines (threshold: 0.01p)
    } else {
        max - min
    };

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = if values.len() > 1 {
                (i as f64 / (values.len() - 1) as f64) * width
            } else {
                width / 2.0 // Center single point
            };
            let y = (1.0 - (val - min) / range).mul_add(2.0f64.mul_add(-padding, height), padding);
            (x, y)
        })
        .collect();

    // Build SVG path with line segments
    let mut path = format!("M {:.2},{:.2}", points[0].0, points[0].1);
    for (x, y) in points.iter().skip(1) {
        use std::fmt::Write;
        write!(path, " L {x:.2},{y:.2}").unwrap();
    }

    path
}

/// Optional: Smooth path using Catmull-Rom to Bezier conversion
#[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
pub fn build_smooth_path(values: &[f64], width: f64, height: f64, padding: f64) -> String {
    use std::fmt::Write;

    if values.len() < 2 {
        return build_path(values, width, height, padding);
    }

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < 0.01 {
        1.0 // Avoid division by zero for flat lines (threshold: 0.01p)
    } else {
        max - min
    };

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = if values.len() > 1 {
                (i as f64 / (values.len() - 1) as f64) * width
            } else {
                width / 2.0 // Center single point
            };
            let y = padding + (1.0 - (val - min) / range) * (height - 2.0 * padding);
            (x, y)
        })
        .collect();

    let mut path = format!("M {:.2},{:.2}", points[0].0, points[0].1);

    // Simple cubic bezier smoothing
    for i in 0..points.len() - 1 {
        let p0 = if i > 0 { points[i - 1] } else { points[i] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() {
            points[i + 2]
        } else {
            p2
        };

        // Catmull-Rom to Bezier control points
        let tension = 6.0;
        let cp1x = p1.0 + (p2.0 - p0.0) / tension;
        let cp1y = p1.1 + (p2.1 - p0.1) / tension;
        let cp2x = p2.0 - (p3.0 - p1.0) / tension;
        let cp2y = p2.1 - (p3.1 - p1.1) / tension;

        write!(
            path,
            " C {cp1x:.2},{cp1y:.2} {cp2x:.2},{cp2y:.2} {:.2},{:.2}",
            p2.0, p2.1
        )
        .unwrap();
    }

    path
}

#[derive(Properties, PartialEq)]
pub struct TraceBannerProps {
    /// Historical price values (31 days × 48 half-hours = ~1488 points)
    pub values: Vec<f64>,

    /// Height in pixels
    #[prop_or(60)]
    pub height: u32,

    /// Stroke color
    #[prop_or_else(|| "var(--color-accent-blue)".to_string())]
    pub color: String,

    /// Stroke width
    #[prop_or(2.0)]
    pub stroke_width: f64,

    /// Use smooth curves instead of line segments
    #[prop_or(true)]
    pub smooth: bool,
}

#[function_component(TraceBanner)]
pub fn trace_banner(props: &TraceBannerProps) -> Html {
    let container_ref = use_node_ref();
    let viewbox_width = use_state(|| 1000.0);

    let viewbox_height = f64::from(props.height);
    let padding = 4.0;

    {
        let viewbox_width = viewbox_width.clone();

        use_effect_with(container_ref.clone(), move |container_ref| {
            let listener = container_ref.cast::<HtmlElement>().map(|container| {
                // Measure initial width
                let width = f64::from(container.client_width());
                if width > 0.0 {
                    viewbox_width.set(width);
                }

                // Setup debounced resize listener (150ms delay)
                let viewbox_width = viewbox_width.clone();
                let container = container.clone();
                create_debounced_resize_listener(
                    move || {
                        let width = f64::from(container.client_width());
                        if width > 0.0 {
                            viewbox_width.set(width);
                        }
                    },
                    150,
                )
            });

            move || drop(listener)
        });
    }

    // Memoize path calculation to prevent recalculation on every render
    let path_data = use_memo(
        (props.values.clone(), *viewbox_width, props.smooth),
        |(values, width, smooth)| {
            if *smooth {
                build_smooth_path(values, *width, viewbox_height, padding)
            } else {
                build_path(values, *width, viewbox_height, padding)
            }
        },
    );

    let viewbox = format!("0 0 {} {}", *viewbox_width, viewbox_height);
    let style = format!("width: 100%; height: {}px; display: block;", props.height);

    html! {
        <svg
            ref={container_ref}
            {viewbox}
            preserveAspectRatio="none"
            {style}
            class="trace-banner"
        >
            <path
                d={(*path_data).clone()}
                fill="none"
                stroke={props.color.clone()}
                stroke-width={props.stroke_width.to_string()}
                stroke-linecap="round"
                stroke-linejoin="round"
                vector-effect="non-scaling-stroke"
            />
        </svg>
    }
}
