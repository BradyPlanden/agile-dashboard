use polars::prelude::*;
use std::num::NonZeroUsize;

pub struct ApiConfig {
    url: Option<String>,
}

impl ApiConfig {
    pub fn new() -> Self {
        Self { url: None }
    }
    fn url(&self) -> String {
        // Return cached value if it exists, otherwise calculate
        self.url.clone().unwrap_or_else(|| {
            "https://api.octopus.energy/v1/products/AGILE-24-10-01/electricity-tariffs/E-1R-AGILE-24-10-01-H/standard-unit-rates/".to_string()
        })
    }
}

/// Fetches API data and stores it as a JSON object
pub async fn get_api_data(config: &ApiConfig) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client.get(config.url()).send().await?;

    response.error_for_status()?.json().await
}

/// Construct a Polars dataframe from a serde JSON object
pub fn construct_dataframe(
    json: &serde_json::value::Value,
    field: &str,
) -> Result<DataFrame, PolarsError> {
    let json_str = serde_json::to_string(&json[field]).expect("Failed to serialize JSON value");

    let df = JsonReader::new(std::io::Cursor::new(json_str.as_bytes()))
        .infer_schema_len(Some(NonZeroUsize::new(100).unwrap())) // Optional: limit rows for schema inference
        .finish()
        .expect("Failed to parse JSON");

    Ok(df)
}
