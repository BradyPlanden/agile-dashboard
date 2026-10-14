//! GB grid carbon intensity from carbonintensity.org.uk.

use chrono::Utc;
use serde::Deserialize;

use crate::models::carbon::{CarbonIntensity, CarbonIntensityData};
use crate::models::error::AppError;
use crate::services::http;

const TODAY_URL: &str = "https://api.carbonintensity.org.uk/intensity/date";

#[derive(Deserialize)]
struct Response {
    data: Vec<CarbonIntensityData>,
}

/// The latest measured intensity and the forecast for the current half hour.
pub async fn fetch_carbon_intensity() -> Result<CarbonIntensity, AppError> {
    let response: Response = http::get_json(TODAY_URL).await?;
    CarbonIntensity::from_periods(&response.data, Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timestamps_with_and_without_seconds() {
        let json = r#"{
            "data": [
                {"from": "2024-01-20T12:00:00Z", "to": "2024-01-20T12:30:00Z",
                 "intensity": {"forecast": 266, "actual": 263, "index": "moderate"}},
                {"from": "2026-01-12T19:30Z", "to": "2026-01-12T20:00Z",
                 "intensity": {"forecast": 142, "index": "very low"}}
            ]
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].intensity.actual, Some(263));
        assert_eq!(response.data[1].intensity.actual, None);
        assert_eq!(
            response.data[1].from.to_rfc3339(),
            "2026-01-12T19:30:00+00:00"
        );
    }
}
