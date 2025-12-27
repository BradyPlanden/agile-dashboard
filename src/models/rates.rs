use super::error::AppError;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Rate {
    pub value_inc_vat: f64,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rates {
    data: Vec<Rate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriceStats {
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub avg: f64,
    pub current: f64,
    pub price_range: String,
}

impl Rates {
    pub fn new(data: Vec<Rate>) -> Self {
        Self { data }
    }

    pub fn current_price(&self) -> Result<f64, AppError> {
        let current_time = Utc::now();

        self.data
            .iter()
            .find(|r| r.valid_from <= current_time && r.valid_to > current_time)
            .map(|r| r.value_inc_vat)
            .ok_or_else(|| AppError::DataError("No current rate found".to_string()))
    }

    pub fn stats(&self) -> Result<PriceStats, AppError> {
        if self.data.is_empty() {
            return Err(AppError::DataError("No data available".to_string()));
        }

        let values: Vec<f64> = self.data.iter().map(|r| r.value_inc_vat).collect();

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;

        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted_values.len() / 2;
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[mid]
        };

        // Use 0.0 if current price is not available, rather than failing the whole stats
        let current = self.current_price().unwrap_or(0.0);

        Ok(PriceStats {
            min,
            max,
            median,
            avg,
            current,
            price_range: format!("{min:.2}p - {max:.2}p"),
        })
    }

    // Filter for today's rates (from midnight today to midnight tomorrow)
    pub fn filter_for_today(&self) -> Vec<Rate> {
        let current_date = Utc::now().date_naive();
        self.data
            .iter()
            .filter(|r| r.valid_from.date_naive() >= current_date)
            .cloned()
            .collect()
    }

    pub fn series_data(&self) -> Result<(Vec<String>, Vec<f64>), AppError> {
        let rates_today = self.filter_for_today();

        // Sort by time just in case
        let mut sorted_rates = rates_today.clone();
        sorted_rates.sort_by(|a, b| a.valid_from.cmp(&b.valid_from));

        let x_data: Vec<String> = sorted_rates
            .iter()
            .map(|r| r.valid_from.format("%Y-%m-%d %H:%M").to_string())
            .collect();

        let y_data: Vec<f64> = sorted_rates.iter().map(|r| r.value_inc_vat).collect();

        Ok((x_data, y_data))
    }
}
