use crate::models::{
    error::AppError,
    rates::{Rate, Rates},
};
use serde::Deserialize;

pub struct ApiConfig {
    url: Option<String>,
}

impl ApiConfig {
    pub fn new() -> Self {
        Self { url: None }
    }
    fn url(&self) -> String {
        self.url.clone().unwrap_or_else(|| {
            "https://api.octopus.energy/v1/products/AGILE-24-10-01/electricity-tariffs/E-1R-AGILE-24-10-01-C/standard-unit-rates/".to_string()
        })
    }
}

#[derive(Deserialize)]
struct ApiResponse {
    results: Vec<Rate>,
}

pub async fn fetch_rates() -> Result<Rates, AppError> {
    let config = ApiConfig::new();
    let client = reqwest::Client::new();

    let response = client
        .get(config.url())
        .send()
        .await
        .map_err(|e| AppError::ApiError(e.to_string()))?;

    let api_response: ApiResponse = response
        .json()
        .await
        .map_err(|e| AppError::ApiError(e.to_string()))?;

    Ok(Rates::new(api_response.results))
}
