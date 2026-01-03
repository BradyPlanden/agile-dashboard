use crate::models::{
    error::AppError,
    rates::{Rate, Rates, TrackerRates},
};
use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::Deserialize;

// CONSTANTS
const BASE_URL: &str = "https://api.octopus.energy/v1/products";
const DEFAULT_AGILE_PRODUCT: &str = "AGILE-24-10-01";
const DEFAULT_TRACKER_PRODUCT: &str = "SILVER-24-10-01";

/// UK electricity distribution regions used by Octopus Energy.
/// Each region corresponds to a Distribution Network Operator (DNO) area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

    /// Returns the region configured for this client.
    pub fn region(&self) -> Region {
        self.region
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
}
