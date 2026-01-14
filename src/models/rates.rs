use super::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    pub value_inc_vat: f64,
    pub value_exc_vat: f64,
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

/// Statistics for a specific day (price range and average only)
#[derive(Debug, Clone, PartialEq)]
pub struct DayStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub price_range: String,
    pub rate_count: usize,
}

/// Combined stats including today/tomorrow + current/next
#[derive(Debug, Clone, PartialEq)]
pub struct DailyStats {
    pub today: DayStats,
    pub tomorrow: Option<DayStats>,
    pub current: f64,
    pub next: f64,
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

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;

        for rate in &self.data {
            let val = rate.value_inc_vat;
            min = min.min(val);
            max = max.max(val);
            sum += val;
        }

        let avg = sum / self.data.len() as f64;
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
        let start_of_today = Utc::now().date_naive();

        let (x_data, y_data): (Vec<_>, Vec<_>) = self
            .data
            .iter()
            .filter(|r| r.valid_from.date_naive() >= start_of_today)
            .map(|r| (r.valid_from.format("%a %H:%M").to_string(), r.value_inc_vat))
            .unzip();

        if x_data.is_empty() {
            return Err(AppError::DataError("No rates for today".to_string()));
        }

        Ok((x_data, y_data))
    }

    /// Extracts values grouped by half-hour slot across multiple days
    /// Returns Vec<Vec<f64>> where outer Vec is 48 half-hour slots,
    /// inner Vec contains values for that slot across all days
    pub fn grouped_by_half_hour_slot(&self) -> Vec<Vec<f64>> {
        use chrono::Timelike;
        use std::collections::HashMap;

        // Group rates by their time-of-day slot (0-47 for 00:00-23:30)
        let mut slots: HashMap<usize, Vec<f64>> = HashMap::new();

        for rate in &self.data {
            let hour = rate.valid_from.hour() as usize;
            let minute = rate.valid_from.minute() as usize;
            let slot = hour * 2 + minute / 30; // 0-47 for half-hour slots

            slots.entry(slot).or_default().push(rate.value_inc_vat);
        }

        // Convert to Vec<Vec<f64>> with 48 entries
        let mut result = vec![vec![]; 48];
        for (slot, values) in slots {
            if slot < 48 {
                result[slot] = values;
            }
        }

        result
    }

    /// Filter rates for a specific date (midnight to midnight UTC)
    fn filter_for_date(&self, date: chrono::NaiveDate) -> Vec<&Rate> {
        use chrono::{Duration, NaiveTime};

        let start_of_day = date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_utc();
        let end_of_day = start_of_day + Duration::days(1);

        self.data
            .iter()
            .filter(|r| r.valid_from >= start_of_day && r.valid_from < end_of_day)
            .collect()
    }

    /// Check if any data exists for a specific date
    pub fn has_data_for_date(&self, date: chrono::NaiveDate) -> bool {
        !self.filter_for_date(date).is_empty()
    }

    /// Compute statistics for a specific date, returns None if no data
    pub fn stats_for_date(&self, date: chrono::NaiveDate) -> Option<DayStats> {
        let filtered_rates = self.filter_for_date(date);

        if filtered_rates.is_empty() {
            return None;
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;

        for rate in &filtered_rates {
            let val = rate.value_inc_vat;
            min = min.min(val);
            max = max.max(val);
            sum += val;
        }

        let avg = sum / filtered_rates.len() as f64;

        Some(DayStats {
            min,
            max,
            avg,
            price_range: format!("{min:.2}p - {max:.2}p"),
            rate_count: filtered_rates.len(),
        })
    }

    /// Get comprehensive daily statistics (today + optional tomorrow)
    pub fn daily_stats(&self) -> Result<DailyStats, AppError> {
        let today = Utc::now().date_naive();
        let tomorrow = today + chrono::Duration::days(1);

        let today_stats = self
            .stats_for_date(today)
            .ok_or_else(|| AppError::DataError("No data for today".to_string()))?;

        let tomorrow_stats = self.stats_for_date(tomorrow);

        let current = self
            .rate_at(Utc::now())
            .map(|r| r.value_inc_vat)
            .unwrap_or(0.0);
        let next = self
            .next_rate(Utc::now())
            .map(|r| r.value_inc_vat)
            .unwrap_or(0.0);

        Ok(DailyStats {
            today: today_stats,
            tomorrow: tomorrow_stats,
            current,
            next,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackerRates {
    data: Vec<Rate>,
}

impl TrackerRates {
    pub fn new(mut data: Vec<Rate>) -> Self {
        data.sort_by_key(|r| r.valid_from);
        Self { data }
    }

    pub fn current_rate(&self) -> Option<&Rate> {
        let now = Utc::now();
        self.data
            .iter()
            .find(|r| r.valid_from <= now && r.valid_to > now)
    }

    pub fn next_day_rate(&self) -> Option<&Rate> {
        let today = Utc::now().date_naive();
        self.data.iter().find(|r| r.valid_from.date_naive() > today)
    }

    pub fn current_price(&self) -> Option<f64> {
        self.current_rate().map(|r| r.value_inc_vat)
    }

    pub fn next_day_price(&self) -> Option<f64> {
        self.next_day_rate().map(|r| r.value_inc_vat)
    }

    pub fn price_difference(&self) -> Option<f64> {
        match (self.current_price(), self.next_day_price()) {
            (Some(current), Some(next)) => Some(next - current),
            _ => None,
        }
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
            value_exc_vat: value / 1.2,
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
                value_exc_vat: 15.0 / 1.2,
                valid_from: valid_from_1,
                valid_to: valid_to_1,
            },
            Rate {
                value_inc_vat: 20.0,
                value_exc_vat: 20.0 / 1.2,
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

    #[test]
    fn test_filter_for_date_boundary() {
        use chrono::NaiveDate;

        // Create rates at 23:30 today and 00:00 tomorrow
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();

        let rate_today_2330 = Rate {
            value_inc_vat: 15.0,
            value_exc_vat: 12.5,
            valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 23, 30, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).unwrap(),
        };

        let rate_tomorrow_0000 = Rate {
            value_inc_vat: 20.0,
            value_exc_vat: 16.67,
            valid_from: Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2024, 1, 16, 0, 30, 0).unwrap(),
        };

        let rates = Rates::new(vec![rate_today_2330.clone(), rate_tomorrow_0000.clone()]);

        // Verify filter_for_date(today) includes only today's rates
        let today_rates = rates.filter_for_date(today);
        assert_eq!(today_rates.len(), 1);
        assert_eq!(today_rates[0].value_inc_vat, 15.0);

        // Verify filter_for_date(tomorrow) includes only tomorrow's rates
        let tomorrow_rates = rates.filter_for_date(tomorrow);
        assert_eq!(tomorrow_rates.len(), 1);
        assert_eq!(tomorrow_rates[0].value_inc_vat, 20.0);
    }

    #[test]
    fn test_stats_for_date_with_data() {
        use chrono::NaiveDate;

        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Create multiple rates for the same day
        let rates = Rates::new(vec![
            Rate {
                value_inc_vat: 10.0,
                value_exc_vat: 8.33,
                valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap(),
                valid_to: Utc.with_ymd_and_hms(2024, 1, 15, 0, 30, 0).unwrap(),
            },
            Rate {
                value_inc_vat: 20.0,
                value_exc_vat: 16.67,
                valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
                valid_to: Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 0).unwrap(),
            },
            Rate {
                value_inc_vat: 15.0,
                value_exc_vat: 12.5,
                valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 23, 30, 0).unwrap(),
                valid_to: Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).unwrap(),
            },
        ]);

        let stats = rates.stats_for_date(today).unwrap();

        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 20.0);
        assert_eq!(stats.avg, 15.0);
        assert_eq!(stats.price_range, "10.00p - 20.00p");
        assert_eq!(stats.rate_count, 3);
    }

    #[test]
    fn test_stats_for_date_no_data() {
        use chrono::NaiveDate;

        let yesterday = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Create rates for yesterday only
        let rates = Rates::new(vec![Rate {
            value_inc_vat: 10.0,
            value_exc_vat: 8.33,
            valid_from: Utc.with_ymd_and_hms(2024, 1, 14, 12, 0, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2024, 1, 14, 12, 30, 0).unwrap(),
        }]);

        // stats_for_date(today) should return None
        assert!(rates.stats_for_date(today).is_none());
    }

    #[test]
    fn test_daily_stats_with_tomorrow() {
        use chrono::{Duration, NaiveDate};

        let today = Utc::now().date_naive();
        let tomorrow = today + Duration::days(1);

        // Create rates for today and tomorrow
        let rates = Rates::new(vec![
            Rate {
                value_inc_vat: 10.0,
                value_exc_vat: 8.33,
                valid_from: today.and_hms_opt(10, 0, 0).unwrap().and_utc(),
                valid_to: today.and_hms_opt(10, 30, 0).unwrap().and_utc(),
            },
            Rate {
                value_inc_vat: 15.0,
                value_exc_vat: 12.5,
                valid_from: tomorrow.and_hms_opt(10, 0, 0).unwrap().and_utc(),
                valid_to: tomorrow.and_hms_opt(10, 30, 0).unwrap().and_utc(),
            },
        ]);

        let daily_stats = rates.daily_stats().unwrap();

        assert_eq!(daily_stats.today.min, 10.0);
        assert!(daily_stats.tomorrow.is_some());
        assert_eq!(daily_stats.tomorrow.unwrap().min, 15.0);
    }

    #[test]
    fn test_daily_stats_without_tomorrow() {
        use chrono::NaiveDate;

        let today = Utc::now().date_naive();

        // Create rates for today only
        let rates = Rates::new(vec![Rate {
            value_inc_vat: 10.0,
            value_exc_vat: 8.33,
            valid_from: today.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            valid_to: today.and_hms_opt(10, 30, 0).unwrap().and_utc(),
        }]);

        let daily_stats = rates.daily_stats().unwrap();

        assert_eq!(daily_stats.today.min, 10.0);
        assert!(daily_stats.tomorrow.is_none());
    }

    #[test]
    fn test_next_price_spanning_midnight() {
        use chrono::NaiveDate;

        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Create rate valid until 23:30
        let rate_today = Rate {
            value_inc_vat: 10.0,
            value_exc_vat: 8.33,
            valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 23, 0, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2024, 1, 15, 23, 30, 0).unwrap(),
        };

        // Create next rate starting 00:00 tomorrow
        let rate_tomorrow = Rate {
            value_inc_vat: 20.0,
            value_exc_vat: 16.67,
            valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 23, 30, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).unwrap(),
        };

        let rates = Rates::new(vec![rate_today, rate_tomorrow.clone()]);

        // Call next_price() at 23:29
        let time_at_23_29 = Utc.with_ymd_and_hms(2024, 1, 15, 23, 29, 0).unwrap();
        let next = rates.next_rate(time_at_23_29).unwrap();

        // Assert returns tomorrow's price
        assert_eq!(next.value_inc_vat, 20.0);
    }

    #[test]
    fn test_has_data_for_date() {
        use chrono::NaiveDate;

        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();

        let rates = Rates::new(vec![Rate {
            value_inc_vat: 10.0,
            value_exc_vat: 8.33,
            valid_from: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 0).unwrap(),
        }]);

        assert!(rates.has_data_for_date(today));
        assert!(!rates.has_data_for_date(tomorrow));
    }
}
