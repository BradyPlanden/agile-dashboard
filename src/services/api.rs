use crate::models::{
    error::AppError,
    rates::{Rate, Rates, TrackerRates},
};
use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

// CONSTANTS
const BASE_URL: &str = "https://api.octopus.energy/v1/products";
const DEFAULT_AGILE_PRODUCT: &str = "AGILE-24-10-01";
const DEFAULT_TRACKER_PRODUCT: &str = "SILVER-24-10-01";

/// UK electricity distribution regions used by Octopus Energy.
/// Each region corresponds to a Distribution Network Operator (DNO) area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Region {
    /// Eastern England
    A,
    /// East Midlands
    B,
    /// London
    #[default]
    C,
    /// Merseyside and North Wales
    D,
    /// West Midlands
    E,
    /// North Eastern England
    F,
    /// North Western England
    G,
    /// Southern England
    H,
    /// South Eastern England
    J,
    /// Southern Wales
    K,
    /// South Western England
    L,
    /// Yorkshire
    M,
    /// Southern Scotland
    N,
    /// Northern Scotland
    P,
}

impl Region {
    /// Returns the single-character code used in API URLs.
    pub fn code(&self) -> &'static str {
        match self {
            Region::A => "A",
            Region::B => "B",
            Region::C => "C",
            Region::D => "D",
            Region::E => "E",
            Region::F => "F",
            Region::G => "G",
            Region::H => "H",
            Region::J => "J",
            Region::K => "K",
            Region::L => "L",
            Region::M => "M",
            Region::N => "N",
            Region::P => "P",
        }
    }

    /// Returns a human-readable description of the region.
    pub fn description(&self) -> &'static str {
        match self {
            Region::A => "Eastern England",
            Region::B => "East Midlands",
            Region::C => "London",
            Region::D => "Merseyside and North Wales",
            Region::E => "West Midlands",
            Region::F => "North Eastern England",
            Region::G => "North Western England",
            Region::H => "Southern England",
            Region::J => "South Eastern England",
            Region::K => "Southern Wales",
            Region::L => "South Western England",
            Region::M => "Yorkshire",
            Region::N => "Southern Scotland",
            Region::P => "Northern Scotland",
        }
    }

    /// All available regions.
    pub fn all() -> &'static [Region] {
        &[
            Region::A,
            Region::B,
            Region::C,
            Region::D,
            Region::E,
            Region::F,
            Region::G,
            Region::H,
            Region::J,
            Region::K,
            Region::L,
            Region::M,
            Region::N,
            Region::P,
        ]
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.code(), self.description())
    }
}

impl std::str::FromStr for Region {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Region::A),
            "B" => Ok(Region::B),
            "C" => Ok(Region::C),
            "D" => Ok(Region::D),
            "E" => Ok(Region::E),
            "F" => Ok(Region::F),
            "G" => Ok(Region::G),
            "H" => Ok(Region::H),
            "J" => Ok(Region::J),
            "K" => Ok(Region::K),
            "L" => Ok(Region::L),
            "M" => Ok(Region::M),
            "N" => Ok(Region::N),
            "P" => Ok(Region::P),
            _ => Err(AppError::ConfigError(format!("Invalid region code: {s}"))),
        }
    }
}

// API CONFIGURATION
/// Configuration for the Octopus Energy API client.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    base_url: String,
    agile_product: String,
    tracker_product: String,
    region: Region,
}

impl ApiConfig {
    /// Creates a builder for constructing an `ApiConfig`.
    pub fn builder() -> ApiConfigBuilder {
        ApiConfigBuilder::default()
    }

    /// Constructs the full URL for Agile tariff rates.
    pub fn agile_url(&self, now: DateTime<Utc>) -> String {
        let base = self.build_tariff_url(&self.agile_product);
        let (from, to) = Self::calculate_period(now);
        format!(
            "{}?period_from={}&period_to={}",
            base,
            from.format("%Y-%m-%dT%H:%M:%SZ"),
            to.format("%Y-%m-%dT%H:%M:%SZ")
        )
    }

    /// Constructs the full URL for historical Agile tariff rates (365 days).
    pub fn agile_url_historical(&self, now: DateTime<Utc>, n_days: i64) -> String {
        let base = self.build_tariff_url(&self.agile_product);
        let (from, to) = Self::calculate_historical_period(now, n_days);
        format!(
            "{}?period_from={}&period_to={}",
            base,
            from.format("%Y-%m-%dT%H:%M:%SZ"),
            to.format("%Y-%m-%dT%H:%M:%SZ")
        )
    }

    /// Constructs the full URL for Tracker tariff rates with date period.
    pub fn tracker_url(&self, now: DateTime<Utc>) -> String {
        let base = self.build_tariff_url(&self.tracker_product);
        let (from, to) = Self::calculate_period(now);
        format!(
            "{}?period_from={}&period_to={}",
            base,
            from.format("%Y-%m-%dT%H:%M:%SZ"),
            to.format("%Y-%m-%dT%H:%M:%SZ")
        )
    }

    fn build_tariff_url(&self, product: &str) -> String {
        format!(
            "{}/{product}/electricity-tariffs/E-1R-{product}-{}/standard-unit-rates/",
            self.base_url,
            self.region.code()
        )
    }

    fn calculate_period(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        let midnight = NaiveTime::MIN;
        let start = now.date_naive().and_time(midnight).and_utc();
        let end = start + Duration::days(2);
        (start, end)
    }

    /// calculate the historical period to acquire agile rates for
    fn calculate_historical_period(
        now: DateTime<Utc>,
        n_days: i64,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let midnight = NaiveTime::MIN;
        let end = now.date_naive().and_time(midnight).and_utc();
        let start = end - Duration::days(n_days);
        (start, end)
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfigBuilder::default().build()
    }
}

// API CONFIGURATION BUILDER
/// Builder for constructing an `ApiConfig` with custom settings.
#[derive(Debug, Default)]
pub struct ApiConfigBuilder {
    base_url: Option<String>,
    agile_product: Option<String>,
    tracker_product: Option<String>,
    region: Option<Region>,
}

impl ApiConfigBuilder {
    /// Sets a custom base URL (primarily for testing).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets the Agile product code.
    pub fn agile_product(mut self, product: impl Into<String>) -> Self {
        self.agile_product = Some(product.into());
        self
    }

    /// Sets the Tracker product code.
    pub fn tracker_product(mut self, product: impl Into<String>) -> Self {
        self.tracker_product = Some(product.into());
        self
    }

    /// Sets the distribution region.
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Builds the `ApiConfig`.
    pub fn build(self) -> ApiConfig {
        ApiConfig {
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.to_string()),
            agile_product: self
                .agile_product
                .unwrap_or_else(|| DEFAULT_AGILE_PRODUCT.to_string()),
            tracker_product: self
                .tracker_product
                .unwrap_or_else(|| DEFAULT_TRACKER_PRODUCT.to_string()),
            region: self.region.unwrap_or_default(),
        }
    }
}

// API RESPONSE TYPES
#[derive(Deserialize, Debug)]
struct ApiResponse<T> {
    results: Vec<T>,
    #[serde(default)]
    next: Option<String>,
    #[serde(default)]
    count: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct ApiRate {
    value_exc_vat: f64,
    value_inc_vat: f64,
    valid_from: DateTime<Utc>,
    valid_to: DateTime<Utc>,
}

impl From<ApiRate> for Rate {
    fn from(r: ApiRate) -> Self {
        Self {
            value_exc_vat: r.value_exc_vat,
            value_inc_vat: r.value_inc_vat,
            valid_from: r.valid_from,
            valid_to: r.valid_to,
        }
    }
}

// OCTOPUS CLIENT
/// HTTP client for the Octopus Energy API.
pub struct OctopusClient {
    http: reqwest::Client,
    config: ApiConfig,
}

impl OctopusClient {
    /// Creates a new client with default configuration.
    pub fn new() -> Result<Self, AppError> {
        Self::with_config(ApiConfig::default())
    }

    /// Creates a new client with the specified configuration.
    pub fn with_config(config: ApiConfig) -> Result<Self, AppError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| AppError::ConfigError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self { http, config })
    }

    /// Returns a reference to the client's configuration.
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }

    /// Fetches Agile tariff rates.
    pub async fn fetch_agile_rates(&self) -> Result<Rates, AppError> {
        let url = self.config.agile_url(Utc::now());

        let rates = self.fetch(&url).await?;
        Ok(Rates::new(rates))
    }

    /// Fetches historical Agile tariff rates (365 days).
    pub async fn fetch_agile_rates_historical(&self) -> Result<Rates, AppError> {
        let url = self.config.agile_url_historical(Utc::now(), 365);

        // Use paginated fetch to get all historical data
        let rates = self.fetch_paginated(&url).await?;
        Ok(Rates::new(rates))
    }

    /// Fetches Tracker tariff rates.
    pub async fn fetch_tracker_rates(&self) -> Result<TrackerRates, AppError> {
        self.fetch_tracker_rates_at(Utc::now()).await
    }

    /// Fetches Tracker tariff rates for a specific point in time.
    pub async fn fetch_tracker_rates_at(
        &self,
        now: DateTime<Utc>,
    ) -> Result<TrackerRates, AppError> {
        let url = self.config.tracker_url(now);

        let rates = self.fetch(&url).await?;
        Ok(TrackerRates::new(rates))
    }

    /// Executes a single fetch attempt.
    async fn fetch(&self, url: &str) -> Result<Vec<Rate>, AppError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| self.classify_error(e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(self.error_for_status(status, &body));
        }

        let api_response: ApiResponse<ApiRate> = response
            .json()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse response: {e}")))?;

        Ok(api_response.results.into_iter().map(Into::into).collect())
    }

    /// Fetches a single page with retry logic for 429 rate limit errors.
    /// Returns the rates and the next page URL if available.
    async fn fetch_page_with_retry(
        &self,
        url: &str,
    ) -> Result<(Vec<Rate>, Option<String>), AppError> {
        use gloo_timers::future::TimeoutFuture;

        let mut retry_delay_ms = 100u32;
        let max_retries = crate::config::Config::MAX_RETRY_ATTEMPTS;

        for attempt in 0..max_retries {
            let response = self
                .http
                .get(url)
                .send()
                .await
                .map_err(|e| self.classify_error(e))?;

            let status = response.status();

            // Handle rate limiting with exponential backoff
            if status.as_u16() == 429 && attempt < max_retries - 1 {
                gloo::console::warn!(format!(
                    "Rate limited, retrying in {}ms (attempt {}/{})",
                    retry_delay_ms,
                    attempt + 1,
                    max_retries
                ));
                TimeoutFuture::new(retry_delay_ms).await;
                retry_delay_ms *= 5; // Exponential backoff: 100ms, 500ms, 2500ms
                continue;
            }

            // Handle other error statuses
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(self.error_for_status(status, &body));
            }

            // Parse successful response
            let api_response: ApiResponse<ApiRate> = response
                .json()
                .await
                .map_err(|e| AppError::ApiError(format!("Failed to parse response: {e}")))?;

            let rates: Vec<Rate> = api_response.results.into_iter().map(Into::into).collect();
            return Ok((rates, api_response.next));
        }

        Err(AppError::RateLimited)
    }

    /// Fetches data across multiple pages, following `next` links.
    /// Returns accumulated data even if later pages fail (partial success).
    async fn fetch_paginated(&self, initial_url: &str) -> Result<Vec<Rate>, AppError> {
        use gloo_timers::future::TimeoutFuture;

        let mut all_rates = Vec::new();
        let mut next_url = Some(initial_url.to_string());
        let mut page = 1;

        while let Some(url) = next_url {
            // Fetch current page with retry logic
            match self.fetch_page_with_retry(&url).await {
                Ok((rates, next)) => {
                    all_rates.extend(rates);
                    next_url = next;

                    // Rate limiting delay between pages (except on last page)
                    if next_url.is_some() {
                        TimeoutFuture::new(crate::config::Config::PAGINATION_DELAY_MS).await;
                    }
                    page += 1;
                }
                Err(e) => {
                    // Return partial data if we have some, otherwise propagate error
                    if all_rates.is_empty() {
                        return Err(e);
                    } else {
                        gloo::console::warn!(format!(
                            "Pagination stopped at page {} with error: {}. Returning {} records.",
                            page,
                            e,
                            all_rates.len()
                        ));
                        break;
                    }
                }
            }
        }

        Ok(all_rates)
    }

    /// Converts a reqwest error into an appropriate AppError.
    fn classify_error(&self, error: reqwest::Error) -> AppError {
        if error.is_timeout() {
            AppError::ApiError(format!("Request timeout: {error}"))
        } else if error.is_request() {
            AppError::ApiError(format!("Request error: {error}"))
        } else {
            AppError::ApiError(format!("Network error: {error}"))
        }
    }

    /// Creates an error based on HTTP status code.
    fn error_for_status(&self, status: reqwest::StatusCode, body: &str) -> AppError {
        match status.as_u16() {
            429 => AppError::RateLimited,
            401 | 403 => AppError::AuthError(format!("Authentication failed: {status}")),
            404 => AppError::NotFound(format!("Resource not found: {body}")),
            400..=499 => AppError::ApiError(format!("Client error {status}: {body}")),
            500..=599 => AppError::ApiError(format!("Server error {status}: {body}")),
            _ => AppError::ApiError(format!("Unexpected status {status}: {body}")),
        }
    }
}

impl Default for OctopusClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

// CONVENIENCE FUNCTIONS
/// Fetches Agile rates using default configuration.
pub async fn fetch_rates() -> Result<Rates, AppError> {
    OctopusClient::new()?.fetch_agile_rates().await
}

/// Fetches historical Agile rates (365 days) using default configuration.
pub async fn fetch_historical_rates() -> Result<Rates, AppError> {
    OctopusClient::new()?.fetch_agile_rates_historical().await
}

/// Fetches Tracker rates using default configuration.
pub async fn fetch_tracker_rates() -> Result<TrackerRates, AppError> {
    OctopusClient::new()?.fetch_tracker_rates().await
}

/// Fetches Agile rates for a specific region.
pub async fn fetch_rates_for_region(region: Region) -> Result<Rates, AppError> {
    let config = ApiConfig::builder().region(region).build();
    OctopusClient::with_config(config)?
        .fetch_agile_rates()
        .await
}

/// Fetches Tracker rates for a specific region.
pub async fn fetch_tracker_rates_for_region(region: Region) -> Result<TrackerRates, AppError> {
    let config = ApiConfig::builder().region(region).build();
    OctopusClient::with_config(config)?
        .fetch_tracker_rates()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_parsing() {
        assert_eq!("C".parse::<Region>().unwrap(), Region::C);
        assert_eq!("c".parse::<Region>().unwrap(), Region::C);
        assert!("X".parse::<Region>().is_err());
    }

    #[test]
    fn test_region_code() {
        assert_eq!(Region::C.code(), "C");
        assert_eq!(Region::M.code(), "M");
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = ApiConfig::builder().build();
        assert_eq!(config.region, Region::C);
    }

    #[test]
    fn test_config_builder_custom_region() {
        let config = ApiConfig::builder().region(Region::M).build();
        assert_eq!(config.region, Region::M);
        assert!(config.agile_url(Utc::now()).contains("-M/"));
    }

    #[test]
    fn test_agile_url_construction() {
        let config = ApiConfig::builder().region(Region::M).build();

        let url = config.agile_url(Utc::now());
        assert!(url.contains("AGILE-24-10-01"));
        assert!(url.contains("-M/"));
    }

    #[test]
    fn test_tracker_url_construction() {
        let config = ApiConfig::builder().region(Region::A).build();
        let now = Utc::now();

        let url = config.tracker_url(now);
        assert!(url.contains("SILVER-24-10-01"));
        assert!(url.contains("-A/"));
        assert!(url.contains("period_from="));
        assert!(url.contains("period_to="));
    }

    #[test]
    fn test_all_regions() {
        let regions = Region::all();
        assert_eq!(regions.len(), 14);

        // Verify no 'I' region (not used by Octopus)
        assert!(!regions.iter().any(|r| r.code() == "I"));
    }

    #[test]
    fn test_api_response_with_pagination() {
        let json = r#"{
            "count": 469,
            "next": "https://api.octopus.energy/v1/products/AGILE-24-10-01/electricity-tariffs/E-1R-AGILE-24-10-01-C/standard-unit-rates/?page=2",
            "results": []
        }"#;

        let response: ApiResponse<ApiRate> = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, Some(469));
        assert!(response.next.is_some());
        assert!(response.results.is_empty());
    }

    #[test]
    fn test_api_response_without_pagination() {
        let json = r#"{"results": []}"#;

        let response: ApiResponse<ApiRate> = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, None);
        assert_eq!(response.next, None);
        assert!(response.results.is_empty());
    }

    #[test]
    fn test_api_response_with_results() {
        let json = r#"{
            "count": 2,
            "next": null,
            "results": [
                {
                    "value_exc_vat": 10.5,
                    "value_inc_vat": 11.025,
                    "valid_from": "2024-01-01T00:00:00Z",
                    "valid_to": "2024-01-01T00:30:00Z"
                },
                {
                    "value_exc_vat": 12.0,
                    "value_inc_vat": 12.6,
                    "valid_from": "2024-01-01T00:30:00Z",
                    "valid_to": "2024-01-01T01:00:00Z"
                }
            ]
        }"#;

        let response: ApiResponse<ApiRate> = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, Some(2));
        assert_eq!(response.next, None);
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].value_exc_vat, 10.5);
        assert_eq!(response.results[1].value_inc_vat, 12.6);
    }
}
