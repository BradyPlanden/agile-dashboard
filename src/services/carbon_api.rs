use crate::models::{
    carbon::{CarbonIntensity, CarbonIntensityData},
    error::AppError,
};
use serde::Deserialize;

const CARBON_API_BASE: &str = "https://api.carbonintensity.org.uk";

/// API response structure from Carbon Intensity API
#[derive(Deserialize, Debug)]
struct CarbonApiResponse {
    data: Vec<CarbonIntensityData>,
}

/// Client for the UK Carbon Intensity API
pub struct CarbonIntensityClient {
    http: reqwest::Client,
    base_url: String,
}

impl CarbonIntensityClient {
    /// Creates a new client with default configuration
    pub fn new() -> Result<Self, AppError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| AppError::ConfigError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url: CARBON_API_BASE.to_string(),
        })
    }

    /// Fetches current and next period carbon intensity for the UK
    pub async fn fetch_current_and_next_intensity(&self) -> Result<CarbonIntensity, AppError> {
        use chrono::Utc;

        crate::services::retry::retry_with_backoff(
            || async {
                let url = format!("{}/intensity/date", self.base_url);

                let response = self
                    .http
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| self.classify_error(e))?;

                let status = response.status();
                if !status.is_success() {
                    let body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "<failed to read error body>".to_string());
                    return Err(self.error_for_status(status, &body));
                }

                let api_response: CarbonApiResponse = response
                    .json()
                    .await
                    .map_err(|e| AppError::ApiError(format!("Failed to parse response: {e}")))?;

                let now = Utc::now();

                // Find most recent period with actual data
                let latest_intensity = api_response
                    .data
                    .iter()
                    .filter(|period| period.to <= now) // Only periods that have ended
                    .filter(|period| period.intensity.actual.is_some()) // Must have actual data
                    .max_by_key(|period| period.to) // Get the most recent one
                    .ok_or_else(|| {
                        AppError::DataError(
                            "No period with actual data found in response".to_string(),
                        )
                    })?
                    .clone();

                // Find the current time
                let next = api_response
                    .data
                    .iter()
                    .find(|period| {
                        // Period that follows now, or period containing now
                        period.from > now || now < period.to
                    })
                    .ok_or_else(|| {
                        AppError::DataError("No next period found in response".to_string())
                    })?
                    .clone();

                Ok(CarbonIntensity::new(latest_intensity, next))
            },
            crate::config::Config::MAX_RETRY_ATTEMPTS,
        )
        .await
    }

    /// Converts a reqwest error into an appropriate `AppError`
    fn classify_error(&self, error: reqwest::Error) -> AppError {
        if error.is_timeout() {
            AppError::ApiError(format!("Request timeout: {error}"))
        } else if error.is_request() {
            AppError::ApiError(format!("Request error: {error}"))
        } else {
            AppError::ApiError(format!("Network error: {error}"))
        }
    }

    /// Creates an error based on HTTP status code
    fn error_for_status(&self, status: reqwest::StatusCode, body: &str) -> AppError {
        match status.as_u16() {
            429 => AppError::RateLimited,
            400..=499 => AppError::ApiError(format!("Client error {status}: {body}")),
            500..=599 => AppError::ApiError(format!("Server error {status}: {body}")),
            _ => AppError::ApiError(format!("Unexpected status {status}: {body}")),
        }
    }
}

/// Convenience function to fetch current and next period carbon intensity
pub async fn fetch_carbon_intensity() -> Result<CarbonIntensity, AppError> {
    CarbonIntensityClient::new()?
        .fetch_current_and_next_intensity()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CarbonIntensityClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_response_parsing() {
        // Test with full timestamp (with seconds)
        let json = r#"{
            "data": [{
                "from": "2024-01-20T12:00:00Z",
                "to": "2024-01-20T12:30:00Z",
                "intensity": {
                    "forecast": 266,
                    "actual": 263,
                    "index": "moderate"
                }
            }]
        }"#;

        let response: CarbonApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].intensity.forecast, 266);
        assert_eq!(response.data[0].intensity.actual, Some(263));
    }

    #[test]
    fn test_api_response_parsing_without_seconds() {
        // Test with timestamp format without seconds (as returned by actual API)
        let json = r#"{
            "data": [{
                "from": "2026-01-12T19:30Z",
                "to": "2026-01-12T20:00Z",
                "intensity": {
                    "forecast": 142,
                    "actual": 133,
                    "index": "moderate"
                }
            }]
        }"#;

        let response: CarbonApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].intensity.forecast, 142);
        assert_eq!(response.data[0].intensity.actual, Some(133));
    }

    #[test]
    fn test_api_response_without_actual() {
        let json = r#"{
            "data": [{
                "from": "2024-01-20T12:00:00Z",
                "to": "2024-01-20T12:30:00Z",
                "intensity": {
                    "forecast": 266,
                    "index": "moderate"
                }
            }]
        }"#;

        let response: CarbonApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data[0].intensity.actual, None);
    }

    #[test]
    fn test_multiple_periods_from_date_endpoint() {
        // Test parsing response similar to /intensity/date endpoint
        let json = r#"{
            "data": [
                {
                    "from": "2026-01-12T00:00Z",
                    "to": "2026-01-12T00:30Z",
                    "intensity": {
                        "forecast": 91,
                        "actual": 95,
                        "index": "moderate"
                    }
                },
                {
                    "from": "2026-01-12T00:30Z",
                    "to": "2026-01-12T01:00Z",
                    "intensity": {
                        "forecast": 91,
                        "actual": 94,
                        "index": "moderate"
                    }
                },
                {
                    "from": "2026-01-12T01:00Z",
                    "to": "2026-01-12T01:30Z",
                    "intensity": {
                        "forecast": 93,
                        "index": "low"
                    }
                }
            ]
        }"#;

        let response: CarbonApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 3);
        assert_eq!(response.data[0].intensity.forecast, 91);
        assert_eq!(response.data[0].intensity.actual, Some(95));
        assert_eq!(response.data[2].intensity.actual, None);
    }
}
