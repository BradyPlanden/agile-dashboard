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
    pub avg: f64,
    pub current: f64,
    pub next: f64,
    pub price_range: String,
}

impl Rates {
    /// Creates a new Rates collection, sorting by valid_from time
    pub fn new(mut data: Vec<Rate>) -> Self {
        data.sort_by_key(|r| r.valid_from);
        Self { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Find the rate valid at a specific time using binary search
    /// Returns None if no rate covers the given time (gap or out of range)
    pub fn rate_at(&self, time: DateTime<Utc>) -> Option<&Rate> {
        // Find first index where valid_from > time
        let idx = self.data.partition_point(|r| r.valid_from <= time);

        // Step back to get the candidate rate
        let rate = self.data.get(idx.checked_sub(1)?)?;

        // Verify the rate actually covers this time (handles gaps)
        (rate.valid_to > time).then_some(rate)
    }

    /// Find the rate immediately following the one valid at the given time
    pub fn next_rate(&self, time: DateTime<Utc>) -> Option<&Rate> {
        let current = self.rate_at(time)?;
        self.rate_at(current.valid_to)
    }

    // Public API using current system time
    pub fn current_rate(&self) -> Result<&Rate, AppError> {
        self.rate_at(Utc::now())
            .ok_or_else(|| AppError::DataError("No current rate found".to_string()))
    }

    pub fn current_price(&self) -> Result<f64, AppError> {
        self.current_rate().map(|r| r.value_inc_vat)
    }

    pub fn next_price(&self) -> Result<f64, AppError> {
        self.next_rate(Utc::now())
            .map(|r| r.value_inc_vat)
            .ok_or_else(|| AppError::DataError("No next rate found".to_string()))
    }

    pub fn stats(&self) -> Result<PriceStats, AppError> {
        self.stats_at(Utc::now())
    }

    // Core functionality
    pub fn stats_at(&self, time: DateTime<Utc>) -> Result<PriceStats, AppError> {
        if self.data.is_empty() {
            return Err(AppError::DataError("No data available".to_string()));
        }

        let values: Vec<f64> = self.data.iter().map(|r| r.value_inc_vat).collect();

        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = values.iter().sum::<f64>() / values.len() as f64;

        let current = self.rate_at(time).map(|r| r.value_inc_vat).unwrap_or(0.0);
        let next = self.next_rate(time).map(|r| r.value_inc_vat).unwrap_or(0.0);

        Ok(PriceStats {
            min,
            max,
            avg,
            current,
            next,
            price_range: format!("{min:.2}p - {max:.2}p"),
        })
    }

    pub fn filter_from(&self, from: DateTime<Utc>) -> impl Iterator<Item = &Rate> {
        self.data.iter().filter(move |r| r.valid_from >= from)
    }

    pub fn filter_for_today(&self) -> Vec<Rate> {
        let start_of_today = Utc::now().date_naive();
        self.data
            .iter()
            .filter(|r| r.valid_from.date_naive() >= start_of_today)
            .cloned()
            .collect()
    }

    pub fn series_data(&self) -> Result<(Vec<String>, Vec<f64>), AppError> {
        let rates_today = self.filter_for_today();

        if rates_today.is_empty() {
            return Err(AppError::DataError("No rates for today".to_string()));
        }

        // Already sorted from construction
        let x_data: Vec<String> = rates_today
            .iter()
            .map(|r| r.valid_from.format("%Y-%m-%d %H:%M").to_string())
            .collect();

        let y_data: Vec<f64> = rates_today.iter().map(|r| r.value_inc_vat).collect();

        Ok((x_data, y_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_rate(hour: u32, value: f64) -> Rate {
        let valid_from = Utc.with_ymd_and_hms(2024, 1, 15, hour, 0, 0).unwrap();
        let valid_to = Utc.with_ymd_and_hms(2024, 1, 15, hour, 30, 0).unwrap();
        Rate {
            value_inc_vat: value,
            valid_from,
            valid_to,
        }
    }

    #[test]
    fn test_rate_at_finds_correct_rate() {
        let rates = Rates::new(vec![
            make_rate(10, 15.0),
            make_rate(11, 20.0),
            make_rate(12, 25.0),
        ]);

        let time = Utc.with_ymd_and_hms(2024, 1, 15, 11, 15, 0).unwrap();
        let rate = rates.rate_at(time).unwrap();

        assert_eq!(rate.value_inc_vat, 20.0);
    }

    #[test]
    fn test_next_rate_finds_following_slot() {
        // Create contiguous rates for this test
        let valid_from_1 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();
        let valid_to_1 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let valid_from_2 = valid_to_1;
        let valid_to_2 = Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap();

        let rates = Rates::new(vec![
            Rate {
                value_inc_vat: 15.0,
                valid_from: valid_from_1,
                valid_to: valid_to_1,
            },
            Rate {
                value_inc_vat: 20.0,
                valid_from: valid_from_2,
                valid_to: valid_to_2,
            },
        ]);

        let time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 15, 0).unwrap();
        let next = rates.next_rate(time).unwrap();

        assert_eq!(next.value_inc_vat, 20.0);
    }

    #[test]
    fn test_rate_at_returns_none_for_gap() {
        let rates = Rates::new(vec![make_rate(10, 15.0)]);

        // Time after the only rate ends
        let time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 45, 0).unwrap();
        assert!(rates.rate_at(time).is_none());
    }
}
