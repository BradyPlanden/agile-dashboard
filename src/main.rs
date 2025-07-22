use chrono::Utc;
use polars_core::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::prelude::*;
use yew_plotly::Plotly;
use yew_plotly::plotly::{Bar, Layout, Plot};

use crate::octoapi::{ApiConfig, construct_dataframe, get_api_data};

mod octoapi;

// Custom error enum
#[derive(Debug)]
enum AppError {
    ApiError(String),
    DataFrameError(String),
    ColumnNotFound(String),
}

impl From<PolarsError> for AppError {
    fn from(err: PolarsError) -> Self {
        AppError::DataFrameError(err.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ApiError(msg) => write!(f, "API Error: {msg}"),
            AppError::DataFrameError(msg) => write!(f, "DataFrame Error: {msg}"),
            AppError::ColumnNotFound(msg) => write!(f, "Column NotFound: {msg}"),
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

                <section class="data-section">
                    <h2>{"Data Summary"}</h2>
                    { render_data_summary(&data_state) }
                </section>

                <section class="chart-section">
                    <h2>{"Energy Price Distribution"}</h2>
                    { render_chart_container(&data_state, &plot_data) }
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

// Compute current price
fn current_price(df: &DataFrame) -> Result<f64, AppError> {
    let current_time = Utc::now().timestamp_millis();

    // Build the mask
    let mask = df.column("valid_from")?.i64()?.lt_eq(current_time)
        & df.column("valid_to")?.i64()?.gt(current_time);

    // Filter the dataframe, compute price
    let current_cost_row = df.filter(&mask)?;
    let price_column = current_cost_row.column("value_inc_vat")?.f64()?;
    let price = match price_column.get(0) {
        Some(p) => p,
        None => {
            console::error_1(&"Price value is null".into());
            return Err(AppError::DataFrameError("Price value is null".to_string()));
        }
    };

    Ok(price)
}

// Convert timestamp column to datetime strings for Plotly
fn extract_datetime_strings(column: &Column) -> Result<Vec<String>, AppError> {
    use web_sys::console;
    let mut values = Vec::new();

    match column.dtype() {
        DataType::Int64 => {
            // Timestamps in milliseconds
            let timestamp_column = column
                .i64()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..timestamp_column.len() {
                if let Some(timestamp_ms) = timestamp_column.get(i) {
                    // Convert milliseconds timestamp to datetime string
                    let datetime = chrono::DateTime::from_timestamp_millis(timestamp_ms)
                        .unwrap_or_else(chrono::Utc::now);

                    // Format for display (you can customize this format)
                    let formatted = datetime.format("%Y-%m-%d %H:%M").to_string();
                    values.push(formatted);
                } else {
                    values.push("Invalid".to_string());
                }
            }
        }
        DataType::Datetime(_, _) => {
            // Already datetime type
            let datetime_column = column
                .datetime()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..datetime_column.len() {
                let val = datetime_column
                    .get(i)
                    .map(|dt| {
                        // Convert to chrono DateTime and format
                        let datetime = chrono::DateTime::from_timestamp_millis(dt / 1_000_000)
                            .unwrap_or_else(chrono::Utc::now);
                        datetime.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "Invalid".to_string());
                values.push(val);
            }
        }
        DataType::String => {
            // Already string, but might need reformatting
            let str_column = column
                .str()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..str_column.len() {
                if let Some(date_str) = str_column.get(i) {
                    // Try to parse and reformat for better display
                    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(date_str) {
                        let formatted = parsed.format("%Y-%m-%d %H:%M").to_string();
                        values.push(formatted);
                    } else {
                        values.push(date_str.to_string());
                    }
                } else {
                    values.push("Invalid".to_string());
                }
            }
        }
        _ => {
            console::error_1(
                &format!(
                    "Unsupported data type for datetime column: {:?}",
                    column.dtype()
                )
                .into(),
            );
            return Err(AppError::DataFrameError(format!(
                "Unsupported data type for datetime column: {:?}",
                column.dtype()
            )));
        }
    }

    Ok(values)
}

// Create Plotly chart from dataframe
fn create_plotly_chart(df: &DataFrame) -> Result<Plot, AppError> {
    let number_of_slots = 48;
    let plot_data = extract_plot_data(&df.head(Some(number_of_slots)))?;

    // Bar chart
    let trace = Bar::new(plot_data.x_data, plot_data.y_data).name("Energy Prices");

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
                            <h3>{"Price Range"}</h3>
                            <p class="summary-value">{&summary.price_range}</p>
                        </div>
                        <div class="summary-item">
                            <h3>{"Average Price"}</h3>
                            <p class="summary-value">{format!("{:.2}p", summary.avg_price)}</p>
                        </div>
                        <div class="summary-item">
                            <h3>{"Current Price"}</h3>
                            <p class="summary-value">{format!("{:.2}p", summary.current_price)}</p>
                        </div>
                        <div class="summary-item">
                            <h3>{"Median Price"}</h3>
                            <p class="summary-value">{format!("{:.2}p", summary.median_price)}</p>
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
    price_range: String,
    avg_price: f64,
    median_price: f64,
    current_price: f64,
}

// Generate data summary from DataFrame
fn generate_data_summary(df: &DataFrame) -> DataSummary {
    let (price_range, median_price, avg_price, current_price) =
        match calculate_price_stats(df, "value_exc_vat") {
            Ok((min, max, median, avg, current)) => {
                (format!("{min:.2}p - {max:.2}p"), median, avg, current)
            }
            Err(e) => {
                console::error_1(&format!("Error calculating price stats: {e}").into());
                ("Unknown".to_string(), 0.0, 0.0, 0.0)
            }
        };

    DataSummary {
        price_range,
        avg_price,
        median_price,
        current_price,
    }
}

// Calculate price statistics
fn calculate_price_stats(
    df: &DataFrame,
    column_name: &str,
) -> Result<(f64, f64, f64, f64, f64), AppError> {
    let s = df
        .column(column_name)
        .map_err(|_| AppError::ColumnNotFound(column_name.to_string()))?;

    let current = current_price(df)?;

    let avg = s
        .mean_reduce()
        .into_value()
        .try_extract::<f64>()
        .map_err(|_| {
            AppError::DataFrameError(format!(
                "Cannot calculate mean value for column {column_name}",
            ))
        })?;
    let median = s
        .median_reduce()
        .unwrap()
        .into_value()
        .try_extract::<f64>()
        .map_err(|_| {
            AppError::DataFrameError(format!(
                "Cannot calculate median value for column {column_name}",
            ))
        })?;
    let min = s
        .min_reduce()
        .unwrap()
        .into_value()
        .try_extract::<f64>()
        .map_err(|_| {
            AppError::DataFrameError(format!(
                "Cannot calculate min value for column {column_name}",
            ))
        })?;
    let max = s
        .max_reduce()
        .unwrap()
        .into_value()
        .try_extract::<f64>()
        .map_err(|_| {
            AppError::DataFrameError(format!(
                "Cannot calculate max value for column {column_name}"
            ))
        })?;

    Ok((min, max, median, avg, current))
}

// Improved plot data extraction with better error handling
#[derive(Debug, Clone)]
struct PlotData {
    x_data: Vec<String>,
    y_data: Vec<f64>,
}

fn extract_plot_data(df: &DataFrame) -> Result<PlotData, AppError> {
    // Validate required columns exist
    let required_columns = ["valid_to", "value_inc_vat"];
    for col in &required_columns {
        if df.column(col).is_err() {
            return Err(AppError::DataFrameError(format!(
                "Required column '{}' not found. Available columns: {:?}",
                col,
                df.get_column_names()
            )));
        }
    }

    let valid_to_column = df
        .column("valid_to")
        .map_err(|e| AppError::DataFrameError(e.to_string()))?;
    let value_inc_vat_column = df
        .column("value_inc_vat")
        .map_err(|e| AppError::DataFrameError(e.to_string()))?;

    // Convert timestamps to datetime strings for Plotly
    let x_data = extract_datetime_strings(valid_to_column)?;
    let y_data = extract_numeric_values(value_inc_vat_column)?;

    // Ensure data lengths match
    if x_data.len() != y_data.len() {
        return Err(AppError::DataFrameError(
            "Mismatch in data lengths between x and y values".to_string(),
        ));
    }

    Ok(PlotData { x_data, y_data })
}

// Extract numeric values from column with error handling
fn extract_numeric_values(column: &Column) -> Result<Vec<f64>, AppError> {
    let mut values = Vec::new();

    match column.dtype() {
        DataType::Float64 => {
            let float_column = column
                .f64()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..float_column.len() {
                let val = float_column.get(i).unwrap_or(0.0);
                values.push(val);
            }
        }
        DataType::Float32 => {
            let float_column = column
                .f32()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..float_column.len() {
                let val = float_column.get(i).unwrap_or(0.0) as f64;
                values.push(val);
            }
        }
        DataType::Int64 => {
            let int_column = column
                .i64()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..int_column.len() {
                let val = int_column.get(i).unwrap_or(0) as f64;
                values.push(val);
            }
        }
        DataType::Int32 => {
            let int_column = column
                .i32()
                .map_err(|e| AppError::DataFrameError(e.to_string()))?;

            for i in 0..int_column.len() {
                let val = int_column.get(i).unwrap_or(0) as f64;
                values.push(val);
            }
        }
        _ => {
            return Err(AppError::DataFrameError(format!(
                "Unsupported data type for numeric column: {:?}",
                column.dtype()
            )));
        }
    }

    Ok(values)
}

fn main() {
    yew::Renderer::<App>::new().render();
}
