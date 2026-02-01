use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

    /// Returns color for display (hex code)
    pub const fn color(&self) -> &'static str {
        match self {
            Self::VeryLow => "#059669",  // dark green
            Self::Low => "#10b981",      // light green
            Self::Moderate => "#f59e0b", // yellow/amber
            Self::High => "#f97316",     // orange
            Self::VeryHigh => "#dc2626", // red
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

    // If string ends with 'Z' but no seconds, parse as UTC naive datetime
    if s.ends_with('Z') {
        let s_without_z = &s[..s.len() - 1];

        // Try with seconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(s_without_z, "%Y-%m-%dT%H:%M:%S") {
            return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
        }

        // Try without seconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(s_without_z, "%Y-%m-%dT%H:%M") {
            return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
        }
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

    /// Check if actual data is available
    pub const fn has_actual(&self) -> bool {
        self.intensity.actual.is_some()
    }
}

/// Container for current and next period carbon intensity data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarbonIntensity {
    pub latest_intensity: CarbonIntensityData,
    pub next: CarbonIntensityData,
}

impl CarbonIntensity {
    pub const fn new(latest_intensity: CarbonIntensityData, next: CarbonIntensityData) -> Self {
        Self {
            latest_intensity,
            next,
        }
    }

    /// Returns the last actual intensity
    pub fn latest_intensity(&self) -> u32 {
        self.latest_intensity.best_intensity()
    }

    /// Returns the forecast intensity for the next period
    pub const fn next_intensity(&self) -> u32 {
        self.next.intensity.forecast
    }

    /// Returns the intensity index for the current period
    pub const fn latest_index(&self) -> IntensityIndex {
        self.latest_intensity.intensity.index
    }

    /// Returns the intensity index for the next period
    pub const fn next_index(&self) -> IntensityIndex {
        self.next.intensity.index
    }

    /// Returns the time range (from, to) for the current period
    pub const fn latest_period(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (self.latest_intensity.from, self.latest_intensity.to)
    }

    /// Returns the time range (from, to) for the next period
    pub const fn next_period(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (self.next.from, self.next.to)
    }

    /// Returns the change in intensity between current and next period
    pub fn intensity_change(&self) -> i32 {
        self.next_intensity().cast_signed() - self.latest_intensity().cast_signed()
    }

    /// Returns whether the current period has actual data
    pub const fn has_actual(&self) -> bool {
        self.latest_intensity.has_actual()
    }
}
