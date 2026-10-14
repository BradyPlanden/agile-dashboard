use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::AppError;

/// Carbon intensity index category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntensityIndex {
    #[serde(rename = "very low")]
    VeryLow,
    Low,
    Moderate,
    High,
    #[serde(rename = "very high")]
    VeryHigh,
}

impl IntensityIndex {
    /// Returns CSS class name for color coding
    pub const fn css_class(&self) -> &'static str {
        match self {
            Self::VeryLow => "intensity-very-low",
            Self::Low => "intensity-low",
            Self::Moderate => "intensity-moderate",
            Self::High => "intensity-high",
            Self::VeryHigh => "intensity-very-high",
        }
    }

    /// Returns human-readable label
    pub const fn label(&self) -> &'static str {
        match self {
            Self::VeryLow => "Very Low",
            Self::Low => "Low",
            Self::Moderate => "Moderate",
            Self::High => "High",
            Self::VeryHigh => "Very High",
        }
    }
}

/// Intensity data for a specific time period
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intensity {
    /// Forecasted carbon intensity (gCO2/kWh)
    pub forecast: u32,

    /// Actual carbon intensity if available (gCO2/kWh)
    #[serde(default)]
    pub actual: Option<u32>,

    /// Intensity category
    pub index: IntensityIndex,
}

/// Carbon intensity data point
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarbonIntensityData {
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub from: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub to: DateTime<Utc>,
    pub intensity: Intensity,
}

/// Custom deserializer for datetime that handles both with and without seconds
fn deserialize_flexible_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use chrono::NaiveDateTime;

    let s: String = serde::Deserialize::deserialize(deserializer)?;

    // Try RFC3339 parsing first (handles most cases)
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // The API often omits seconds ("2026-01-12T19:30Z"), which RFC 3339 requires
    if let Some(naive) = s.strip_suffix('Z').and_then(|s| {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M"))
            .ok()
    }) {
        return Ok(naive.and_utc());
    }

    Err(serde::de::Error::custom(format!(
        "Failed to parse datetime '{s}'"
    )))
}

impl CarbonIntensityData {
    /// Get the best available intensity value (actual if present, otherwise forecast)
    pub fn best_intensity(&self) -> u32 {
        self.intensity.actual.unwrap_or(self.intensity.forecast)
    }
}

/// Container for current and next period carbon intensity data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarbonIntensity {
    pub latest: CarbonIntensityData,
    pub next: CarbonIntensityData,
}

impl CarbonIntensity {
    pub const fn new(latest: CarbonIntensityData, next: CarbonIntensityData) -> Self {
        Self { latest, next }
    }

    /// Picks the most recent measured period and the forecast for `now` from a day of periods.
    pub fn from_periods(
        periods: &[CarbonIntensityData],
        now: DateTime<Utc>,
    ) -> Result<Self, AppError> {
        let latest = periods
            .iter()
            .filter(|period| period.to <= now && period.intensity.actual.is_some())
            .max_by_key(|period| period.to)
            .ok_or(AppError::MissingData(
                "no measured carbon intensity yet today",
            ))?;

        let next = periods
            .iter()
            .find(|period| period.from <= now && now < period.to)
            .or_else(|| periods.iter().find(|period| period.from > now))
            .ok_or(AppError::MissingData(
                "no carbon intensity forecast for now",
            ))?;

        Ok(Self::new(latest.clone(), next.clone()))
    }

    /// Returns the last actual intensity value
    pub fn latest_value(&self) -> u32 {
        self.latest.best_intensity()
    }

    /// Returns the forecast intensity for the next period
    pub const fn next_forecast(&self) -> u32 {
        self.next.intensity.forecast
    }

    /// Returns the intensity index for the current period
    pub const fn latest_index(&self) -> IntensityIndex {
        self.latest.intensity.index
    }

    /// Returns the intensity index for the next period
    pub const fn next_index(&self) -> IntensityIndex {
        self.next.intensity.index
    }

    /// Returns the time range (from, to) for the current period
    pub const fn latest_period(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (self.latest.from, self.latest.to)
    }

    /// Returns the time range (from, to) for the next period
    pub const fn next_period(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (self.next.from, self.next.to)
    }

    /// Returns the change in intensity between current and next period
    pub fn intensity_change(&self) -> i32 {
        self.next_forecast().cast_signed() - self.latest_value().cast_signed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn period(from_min: i64, actual: Option<u32>, forecast: u32) -> CarbonIntensityData {
        let midnight = Utc.with_ymd_and_hms(2026, 1, 12, 0, 0, 0).unwrap();
        CarbonIntensityData {
            from: midnight + chrono::Duration::minutes(from_min),
            to: midnight + chrono::Duration::minutes(from_min + 30),
            intensity: Intensity {
                forecast,
                actual,
                index: IntensityIndex::Moderate,
            },
        }
    }

    #[test]
    fn picks_latest_measured_and_current_forecast() {
        let periods = [
            period(0, Some(95), 91),
            period(30, Some(94), 91),
            period(60, None, 93),
            period(90, None, 120),
        ];
        let now = Utc.with_ymd_and_hms(2026, 1, 12, 1, 10, 0).unwrap();

        let carbon = CarbonIntensity::from_periods(&periods, now).unwrap();
        assert_eq!(carbon.latest_value(), 94);
        assert_eq!(carbon.next_forecast(), 93);
        assert_eq!(carbon.intensity_change(), -1);
    }

    #[test]
    fn falls_back_to_the_next_period_when_none_covers_now() {
        let periods = [period(0, Some(95), 91), period(60, None, 120)];
        let now = Utc.with_ymd_and_hms(2026, 1, 12, 0, 45, 0).unwrap();

        let carbon = CarbonIntensity::from_periods(&periods, now).unwrap();
        assert_eq!(carbon.next_forecast(), 120);
    }

    #[test]
    fn errors_without_any_measured_period() {
        let periods = [period(0, None, 91)];
        let now = Utc.with_ymd_and_hms(2026, 1, 12, 0, 45, 0).unwrap();

        assert!(CarbonIntensity::from_periods(&periods, now).is_err());
    }
}
