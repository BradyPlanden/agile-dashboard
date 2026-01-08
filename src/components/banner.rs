use gloo::events::EventListener;
use web_sys::HtmlElement;
use yew::prelude::*;

/// Computes mean for each time index across all 48 elements
/// Input: 48 x N flattened or nested data (where N is number of days)
/// Output: N mean values (averaged across all 48 half-hour slots per day)
pub fn compute_means(data: &[Vec<f64>]) -> Vec<f64> {
    if data.is_empty() {
        return vec![];
    }

    // Find the maximum length to determine how many days we have
    let max_len = data.iter().map(|v| v.len()).max().unwrap_or(0);
    if max_len == 0 {
        return vec![];
    }

    (0..max_len)
        .map(|i| {
            // Only sum values where the series has data at index i
            let (sum, count) = data.iter().fold((0.0, 0), |(sum, count), series| {
                if let Some(&value) = series.get(i) {
                    (sum + value, count + 1)
                } else {
                    (sum, count)
                }
            });

            // Average only across slots that have data for this day
            if count > 0 { sum / count as f64 } else { 0.0 }
        })
        .collect()
}

/// Generates SVG path data from values
fn build_path(values: &[f64], width: f64, height: f64, padding: f64) -> String {
    if values.is_empty() {
        return String::new();
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < f64::EPSILON {
        1.0 // Avoid division by zero for flat lines
    } else {
        max - min
    };

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = (i as f64 / (values.len() - 1) as f64) * width;
            let y = padding + (1.0 - (val - min) / range) * (height - 2.0 * padding);
            (x, y)
        })
        .collect();

    // Build SVG path with line segments
    let mut path = format!("M {:.2},{:.2}", points[0].0, points[0].1);
    for (x, y) in points.iter().skip(1) {
        path.push_str(&format!(" L {:.2},{:.2}", x, y));
    }

    path
}

/// Optional: Smooth path using Catmull-Rom to Bezier conversion
fn build_smooth_path(values: &[f64], width: f64, height: f64, padding: f64) -> String {
    if values.len() < 2 {
        return build_path(values, width, height, padding);
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        max - min
    };

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = (i as f64 / (values.len() - 1) as f64) * width;
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

        path.push_str(&format!(
            " C {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
            cp1x, cp1y, cp2x, cp2y, p2.0, p2.1
        ));
    }

    path
}

#[derive(Properties, PartialEq)]
pub struct TraceBannerProps {
    /// Pre-computed mean values (365 points)
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

    let viewbox_height = props.height as f64;
    let padding = 4.0;

    {
        let container_ref = container_ref.clone();
        let viewbox_width = viewbox_width.clone();

        use_effect_with(container_ref.clone(), move |container_ref| {
            let listener = container_ref.cast::<HtmlElement>().and_then(|container| {
                // Measure initial width
                let width = container.client_width() as f64;
                if width > 0.0 {
                    viewbox_width.set(width);
                }

                // Setup resize listener
                let viewbox_width = viewbox_width.clone();
                Some(EventListener::new(
                    &web_sys::window().unwrap(),
                    "resize",
                    move |_| {
                        let width = container.client_width() as f64;
                        if width > 0.0 {
                            viewbox_width.set(width);
                        }
                    },
                ))
            });

            move || drop(listener)
        });
    }

    let path_data = if props.smooth {
        build_smooth_path(&props.values, *viewbox_width, viewbox_height, padding)
    } else {
        build_path(&props.values, *viewbox_width, viewbox_height, padding)
    };

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
                d={path_data}
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
