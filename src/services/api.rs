use crate::models::{
    error::AppError,
    rates::{Rate, Rates, TrackerRates},
};
use crate::services::error::{build_http_client, error_for_status, send_with_timeout};
use crate::utils::time::{london_date, london_midnight_utc};
use chrono::{DateTime, Days, Utc};
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
    pub const fn code(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::P => "P",
        }
    }

    /// Returns a human-readable description of the region.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::A => "Eastern England",
            Self::B => "East Midlands",
            Self::C => "London",
            Self::D => "Merseyside and North Wales",
            Self::E => "West Midlands",
            Self::F => "North Eastern England",
            Self::G => "North Western England",
            Self::H => "Southern England",
            Self::J => "South Eastern England",
            Self::K => "Southern Wales",
            Self::L => "South Western England",
            Self::M => "Yorkshire",
            Self::N => "Southern Scotland",
            Self::P => "Northern Scotland",
        }
    }

    /// All available regions.
    pub const fn all() -> &'static [Self] {
        &[
            Self::A,
            Self::B,
            Self::C,
            Self::D,
            Self::E,
            Self::F,
            Self::G,
            Self::H,
            Self::J,
            Self::K,
            Self::L,
            Self::M,
            Self::N,
            Self::P,
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
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "E" => Ok(Self::E),
            "F" => Ok(Self::F),
            "G" => Ok(Self::G),
            "H" => Ok(Self::H),
            "J" => Ok(Self::J),
            "K" => Ok(Self::K),
            "L" => Ok(Self::L),
            "M" => Ok(Self::M),
            "N" => Ok(Self::N),
            "P" => Ok(Self::P),
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

    /// Constructs the full URL for historical Agile tariff rates.
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
        let today = london_date(now);
        let end_date = today
            .checked_add_days(Days::new(2))
            .expect("two days after today should be valid");
        let start = london_midnight_utc(today);
        let end = london_midnight_utc(end_date);
        (start, end)
    }

    /// Calculate the historical period to acquire agile rates for
    fn calculate_historical_period(
        now: DateTime<Utc>,
        n_days: i64,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let today = london_date(now);
        let day_count = u64::try_from(n_days).expect("historical day count must be non-negative");
        let end = london_midnight_utc(today);
        let start_date = today
            .checked_sub_days(Days::new(day_count))
            .expect("historical start date should be valid");
        let start = london_midnight_utc(start_date);
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
    /// Sets the distribution region.
    pub const fn region(mut self, region: Region) -> Self {
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
    #[allow(dead_code)]
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
///
/// Uses a shared `reqwest::Client` for connection pooling.
static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(build_http_client);

pub struct OctopusClient {
    http: reqwest::Client,
    config: ApiConfig,
}

pub struct HistoricalRatesFetchResult {
    pub rates: Rates,
    pub complete: bool,
}

struct PageFetchResult {
    rates: Vec<Rate>,
    next_url: Option<String>,
    next_rejected: bool,
}

struct PaginatedFetchResult {
    rates: Vec<Rate>,
    complete: bool,
}

impl OctopusClient {
    /// Creates a new client with the specified configuration.
    pub fn with_config(config: ApiConfig) -> Self {
        Self {
            http: HTTP_CLIENT.clone(),
            config,
        }
    }

    /// Fetches Agile tariff rates.
    pub async fn fetch_agile_rates(&self) -> Result<Rates, AppError> {
        let url = self.config.agile_url(Utc::now());

        let rates = self.fetch(&url).await?;
        Ok(Rates::new(rates))
    }

    /// Fetches historical Agile tariff rates (31 days), publishing intermediate snapshots.
    pub async fn fetch_agile_rates_historical_progressive<F>(
        &self,
        on_progress: F,
    ) -> Result<HistoricalRatesFetchResult, AppError>
    where
        F: FnMut(Rates),
    {
        let url = self.config.agile_url_historical(Utc::now(), 31);

        let result = self.fetch_paginated_with_progress(&url, on_progress).await?;
        Ok(HistoricalRatesFetchResult {
            rates: Rates::new(result.rates),
            complete: result.complete,
        })
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
        let response = self.http.get(url);
        let response = send_with_timeout(response).await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(error_for_status(status, &body));
        }

        let api_response: ApiResponse<ApiRate> = response
            .json()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse response: {e}")))?;

        Ok(api_response.results.into_iter().map(Into::into).collect())
    }

    /// Executes a single page fetch with retry logic for rate-limited responses.
    /// Returns the rates and the next page URL if available.
    async fn fetch_page_with_retry(
        &self,
        url: &str,
    ) -> Result<PageFetchResult, AppError> {
        crate::services::retry::retry_with_backoff(
            || async {
                let response = self.http.get(url);
                let response = send_with_timeout(response).await?;

                let status = response.status();

                // Handle non-429 error statuses immediately
                if !status.is_success() && status.as_u16() != 429 {
                    let body = response.text().await.unwrap_or_default();
                    return Err(error_for_status(status, &body));
                }

                // Successful response
                if status.is_success() {
                    let api_response: ApiResponse<ApiRate> =
                        response.json().await.map_err(|e| {
                            AppError::ApiError(format!("Failed to parse response: {e}"))
                        })?;

                    let next_page = sanitize_next_url(api_response.next);
                    let rates: Vec<Rate> =
                        api_response.results.into_iter().map(Into::into).collect();
                    return Ok(PageFetchResult {
                        rates,
                        next_url: next_page.url,
                        next_rejected: next_page.rejected,
                    });
                }

                // 429 — will be retried by retry_with_backoff
                Err(AppError::RateLimited)
            },
            crate::config::Config::MAX_RETRY_ATTEMPTS,
        )
        .await
    }

    /// Fetches data across multiple pages, following `next` links.
    /// Publishes accumulated snapshots as pages arrive and returns partial data if later pages fail.
    async fn fetch_paginated_with_progress<F>(
        &self,
        initial_url: &str,
        mut on_progress: F,
    ) -> Result<PaginatedFetchResult, AppError>
    where
        F: FnMut(Rates),
    {
        use gloo_timers::future::TimeoutFuture;

        let mut all_rates = Vec::new();
        let mut next_url = Some(initial_url.to_string());
        let mut page = 1;
        let mut complete = true;

        while let Some(url) = next_url {
            // Fetch current page with retry logic
            match self.fetch_page_with_retry(&url).await {
                Ok(page_result) => {
                    all_rates.extend(page_result.rates);
                    on_progress(Rates::new(all_rates.clone()));

                    if page_result.next_rejected {
                        warn(&format!(
                            "Pagination stopped at page {} because the next link was rejected. Returning {} records.",
                            page,
                            all_rates.len()
                        ));
                        complete = false;
                        break;
                    }

                    next_url = page_result.next_url;

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
                    }
                    warn(&format!(
                        "Pagination stopped at page {} with error: {}. Returning {} records.",
                        page,
                        e,
                        all_rates.len()
                    ));
                    complete = false;
                    break;
                }
            }
        }

        Ok(PaginatedFetchResult {
            rates: all_rates,
            complete,
        })
    }
}

/// Validates that a pagination URL targets the expected Octopus API host.
///
/// Prevents following arbitrary redirects from compromised or buggy API responses.
fn validate_next_url(url: Option<&str>) -> Option<String> {
    match url {
        None => None,
        Some(raw) => match reqwest::Url::parse(raw) {
            Ok(parsed)
                if parsed.scheme() == "https"
                    && parsed.host_str() == Some("api.octopus.energy") =>
            {
                Some(parsed.into())
            }
            Ok(_) | Err(_) => {
                warn(&format!(
                    "Rejecting pagination URL with unexpected origin: {raw}"
                ));
                None
            }
        },
    }
}

struct SanitizedNextUrl {
    url: Option<String>,
    rejected: bool,
}

fn sanitize_next_url(raw_next: Option<String>) -> SanitizedNextUrl {
    let url = validate_next_url(raw_next.as_deref());
    let rejected = raw_next.is_some() && url.is_none();
    SanitizedNextUrl { url, rejected }
}

fn warn(message: &str) {
    #[cfg(target_arch = "wasm32")]
    gloo::console::warn!(message);

    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{message}");
}

// CONVENIENCE FUNCTIONS
/// Fetches historical Agile rates (31 days), publishing progressive snapshots as pages arrive.
pub async fn fetch_historical_rates_progressive<F>(
    on_progress: F,
) -> Result<HistoricalRatesFetchResult, AppError>
where
    F: FnMut(Rates),
{
    OctopusClient::with_config(ApiConfig::default())
        .fetch_agile_rates_historical_progressive(on_progress)
        .await
}

/// Fetches Agile rates for a specific region.
pub async fn fetch_rates_for_region(region: Region) -> Result<Rates, AppError> {
    OctopusClient::with_config(ApiConfig::builder().region(region).build())
        .fetch_agile_rates()
        .await
}

/// Fetches Tracker rates for a specific region.
pub async fn fetch_tracker_rates_for_region(region: Region) -> Result<TrackerRates, AppError> {
    OctopusClient::with_config(ApiConfig::builder().region(region).build())
        .fetch_tracker_rates()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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
    fn test_calculate_period_uses_london_midnight_in_bst() {
        let now = Utc.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).unwrap();
        let (start, end) = ApiConfig::calculate_period(now);

        assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 29, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 3, 30, 23, 0, 0).unwrap());
    }

    #[test]
    fn test_calculate_historical_period_uses_london_midnight_in_bst() {
        let now = Utc.with_ymd_and_hms(2026, 3, 30, 12, 0, 0).unwrap();
        let (start, end) = ApiConfig::calculate_historical_period(now, 1);

        assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 29, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 3, 29, 23, 0, 0).unwrap());
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

    #[test]
    fn test_validate_next_url_accepts_known_host() {
        let url = Some(
            "https://api.octopus.energy/v1/products/AGILE-24-10-01/standard-unit-rates/?page=2"
                .to_string(),
        );
        let result = validate_next_url(url.as_deref());
        assert!(result.is_some());
    }

    #[test]
    fn test_validate_next_url_rejects_unknown_host() {
        let url = Some("https://evil.example.com/api/data".to_string());
        let result = validate_next_url(url.as_deref());
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_next_url_rejects_http() {
        let url = Some("http://api.octopus.energy/v1/products/?page=2".to_string());
        let result = validate_next_url(url.as_deref());
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_next_url_none() {
        let result = validate_next_url(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_sanitize_next_url_marks_rejected_links_incomplete() {
        let result = sanitize_next_url(Some("https://evil.example.com/page=2".to_string()));
        assert!(result.url.is_none());
        assert!(result.rejected);
    }

    #[test]
    fn test_sanitize_next_url_allows_absent_links() {
        let result = sanitize_next_url(None);
        assert!(result.url.is_none());
        assert!(!result.rejected);
    }
}
