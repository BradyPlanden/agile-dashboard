use polars::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::prelude::*;
use yew_plotly::Plotly;
use yew_plotly::plotly::{Bar, Layout, Plot};

use crate::octoapi::{ApiConfig, construct_dataframe, get_api_data};

mod octoapi;

// Custom error type
#[derive(Debug)]
enum AppError {
    ApiError(String),
    DataFrameError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ApiError(msg) => write!(f, "API Error: {msg}"),
            AppError::DataFrameError(msg) => write!(f, "DataFrame Error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

// State management
#[derive(Clone, PartialEq)]
enum DataState {
    Loading,
    Loaded(DataFrame),
    Error(String),
}

#[function_component]
fn App() -> Html {
    let data_state = use_state(|| DataState::Loading);
    let plot_data = use_state(|| None::<Plot>);

    // Async data loading
    {
        let data_state = data_state.clone();
        let plot_data = plot_data.clone();

        use_effect(move || {
            spawn_local(async move {
                match load_data().await {
                    Ok(dataframe) => match create_plotly_chart(&dataframe) {
                        Ok(plot) => {
                            plot_data.set(Some(plot));
                            data_state.set(DataState::Loaded(dataframe));
                        }
                        Err(e) => {
                            console::error_1(&format!("Error creating plot: {e}").into());
                            data_state.set(DataState::Error(e.to_string()));
                        }
                    },
                    Err(e) => {
                        console::error_1(&format!("Error loading data: {e}").into());
                        data_state.set(DataState::Error(e.to_string()));
                    }
                }
            });
            || () // Cleanup function
        });
    }

    html! {
        <div class="app-container">
            <header class="app-header">
                <h1>{"Octopus Agile Dashboard"}</h1>
            </header>

            <main class="app-main">
                <section class="status-section">
                    <h2>{"API Status"}</h2>
                    { render_status(&data_state) }
                </section>

                <section class="chart-section">
                    <h2>{"Energy Price Distribution"}</h2>
                    { render_chart_container(&data_state, &plot_data) }
                </section>

                <section class="data-section">
                    <h2>{"Data Summary"}</h2>
                    { render_data_summary(&data_state) }
                </section>
            </main>

            <style>
                {include_str!("style.css")}
            </style>
        </div>
    }
}

// Async data loading function
async fn load_data() -> Result<DataFrame, AppError> {
    let api_config = ApiConfig::new();

    let result = get_api_data(&api_config)
        .await
        .map_err(|e| AppError::ApiError(e.to_string()))?;

    let dataframe = construct_dataframe(&result, "results")
        .map_err(|e| AppError::DataFrameError(e.to_string()))?;

    Ok(dataframe)
}

// Create Plotly chart from DataFrame
fn create_plotly_chart(df: &DataFrame) -> Result<Plot, AppError> {
    let plot_data = extract_plot_data(df)?;

    // Bar chart
    let trace = Bar::new(plot_data.x_data, plot_data.y_data).name("Energy Prices");

    // Layout
    let layout = Layout::new()
        .title("Energy Price Distribution".into())
        .x_axis(
            yew_plotly::plotly::layout::Axis::new()
                .title("Time".into())
                .tick_angle(-45.0),
        )
        .y_axis(yew_plotly::plotly::layout::Axis::new().title("Price (pence/kWh)".into()))
        .height(500)
        .margin(yew_plotly::plotly::layout::Margin::new().bottom(100));

    // Create plot
    let mut plot = Plot::new();
    plot.add_trace(trace);
    plot.set_layout(layout);

    Ok(plot)
}

// Render status component
fn render_status(state: &DataState) -> Html {
    match state {
        DataState::Loading => html! {
            <div class="status loading">
                <div class="spinner"></div>
                <p>{"Loading data..."}</p>
            </div>
        },
        DataState::Loaded(_) => html! {
            <div class="status success">
                <p>{"✅ Data loaded successfully"}</p>
            </div>
        },
        DataState::Error(msg) => html! {
            <div class="status error">
                <p>{"❌ Error: "}{msg}</p>
            </div>
        },
    }
}

// Render chart container
fn render_chart_container(state: &DataState, plot_data: &Option<Plot>) -> Html {
    match state {
        DataState::Loading => html! {
            <div class="chart-placeholder">
                <div class="spinner"></div>
                <p>{"Loading chart..."}</p>
            </div>
        },
        DataState::Loaded(_) => {
            if let Some(plot) = plot_data {
                html! {
                    <div class="chart-container">
                        <Plotly plot={plot.clone()} />
                    </div>
                }
            } else {
                html! {
                    <div class="chart-loading">
                        <p>{"Preparing chart..."}</p>
                    </div>
                }
            }
        }
        DataState::Error(_) => html! {
            <div class="chart-error">
                <p>{"Unable to render chart due to data loading error"}</p>
            </div>
        },
    }
}

// Render data summary
fn render_data_summary(state: &DataState) -> Html {
    match state {
        DataState::Loading => html! {
            <div class="data-summary loading">
                <p>{"Loading summary..."}</p>
            </div>
        },
        DataState::Loaded(df) => {
            let summary = generate_data_summary(df);
            html! {
                <div class="data-summary">
                    <div class="summary-grid">
                        <div class="summary-item">
                            <h3>{"Total Records"}</h3>
                            <p class="summary-value">{summary.total_records}</p>
                        </div>
                        <div class="summary-item">
                            <h3>{"Date Range"}</h3>
                            <p class="summary-value">{&summary.date_range}</p>
                        </div>
                        <div class="summary-item">
                            <h3>{"Price Range"}</h3>
                            <p class="summary-value">{&summary.price_range}</p>
                        </div>
                        <div class="summary-item">
                            <h3>{"Average Price"}</h3>
                            <p class="summary-value">{format!("{:.2}p", summary.avg_price)}</p>
                        </div>
                    </div>
                </div>
            }
        }
        DataState::Error(_) => html! {
            <div class="data-summary error">
                <p>{"No summary available"}</p>
            </div>
        },
    }
}

// Data summary structure
#[derive(Debug, Clone)]
struct DataSummary {
    total_records: usize,
    date_range: String,
    price_range: String,
    avg_price: f64,
}

// Generate data summary from DataFrame
fn generate_data_summary(df: &DataFrame) -> DataSummary {
    let total_records = df.height();

    let date_range = if let Ok(valid_to_col) = df.column("valid_to") {
        // This is a simplified example - you'd want proper date parsing
        format!("{} entries", valid_to_col.len())
    } else {
        "Unknown".to_string()
    };

    let (price_range, avg_price) = if let Ok(value_col) = df.column("value_exc_vat") {
        match calculate_price_stats(value_col) {
            Ok((min, max, avg)) => (format!("{min:.2}p - {max:.2}p"), avg),
            Err(_) => ("Unknown".to_string(), 0.0),
        }
    } else {
        ("Unknown".to_string(), 0.0)
    };

    DataSummary {
        total_records,
        date_range,
        price_range,
        avg_price,
    }
}

// Calculate price statistics
fn calculate_price_stats(series: &Column) -> Result<(f64, f64, f64), AppError> {
    let values = extract_numeric_values(series)?;

    if values.is_empty() {
        return Err(AppError::DataFrameError(
            "No valid numeric values found".to_string(),
        ));
    }

    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let avg = values.iter().sum::<f64>() / values.len() as f64;

    Ok((min, max, avg))
}

// Improved plot data extraction with better error handling
#[derive(Debug, Clone)]
struct PlotData {
    x_data: Vec<String>,
    y_data: Vec<f64>,
}

fn extract_plot_data(df: &DataFrame) -> Result<PlotData, AppError> {
    // Validate required columns exist
    let required_columns = ["valid_to", "value_exc_vat"];
    for col in &required_columns {
        if df.column(col).is_err() {
            return Err(AppError::DataFrameError(format!(
                "Required column '{}' not found. Available columns: {:?}",
                col,
                df.get_column_names()
            )));
        }
    }

    let valid_to_series = df
        .column("valid_to")
        .map_err(|e| AppError::DataFrameError(e.to_string()))?;
    let value_exc_vat_series = df
        .column("value_exc_vat")
        .map_err(|e| AppError::DataFrameError(e.to_string()))?;

    let x_data = extract_string_values(valid_to_series)?;
    let y_data = extract_numeric_values(value_exc_vat_series)?;

    // Ensure data lengths match
    if x_data.len() != y_data.len() {
        return Err(AppError::DataFrameError(
            "Mismatch in data lengths between x and y values".to_string(),
        ));
    }

    Ok(PlotData { x_data, y_data })
}

// Extract string values from series with proper error handling
fn extract_string_values(series: &Column) -> Result<Vec<String>, AppError> {
    let mut values = Vec::new();

    match series.dtype() {
        DataType::String => {
            let str_series = series
                .str()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..str_series.len() {
                let val = str_series
                    .get(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "null".to_string());
                values.push(val);
            }
        }
        DataType::Datetime(_, _) => {
            let datetime_series = series
                .datetime()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..datetime_series.len() {
                let val = datetime_series
                    .get(i)
                    .map(|dt| format!("{dt}"))
                    .unwrap_or_else(|| "null".to_string());
                values.push(val);
            }
        }
        _ => {
            // Fallback: convert other types to string
            for i in 0..series.len() {
                values.push(format!("Item {i}"));
            }
        }
    }

    Ok(values)
}

// Extract numeric values from series with proper error handling
fn extract_numeric_values(series: &Column) -> Result<Vec<f64>, AppError> {
    let mut values = Vec::new();

    match series.dtype() {
        DataType::Float64 => {
            let float_series = series
                .f64()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..float_series.len() {
                let val = float_series.get(i).unwrap_or(0.0);
                values.push(val);
            }
        }
        DataType::Float32 => {
            let float_series = series
                .f32()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..float_series.len() {
                let val = float_series.get(i).unwrap_or(0.0) as f64;
                values.push(val);
            }
        }
        DataType::Int64 => {
            let int_series = series
                .i64()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..int_series.len() {
                let val = int_series.get(i).unwrap_or(0) as f64;
                values.push(val);
            }
        }
        DataType::Int32 => {
            let int_series = series
                .i32()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..int_series.len() {
                let val = int_series.get(i).unwrap_or(0) as f64;
                values.push(val);
            }
        }
        _ => {
            return Err(AppError::DataFrameError(format!(
                "Unsupported data type for numeric column: {:?}",
                series.dtype()
            )));
        }
    }

    Ok(values)
}

fn main() {
    yew::Renderer::<App>::new().render();
}
