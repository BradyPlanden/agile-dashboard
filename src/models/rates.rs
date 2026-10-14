use std::fmt;

use chrono::{DateTime, Days, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::utils::time::london_midnight_utc;

/// One unit rate, as returned by the Octopus API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    pub value_inc_vat: f64,
    pub value_exc_vat: f64,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
}

impl Rate {
    /// Whether `time` falls within this rate's `[valid_from, valid_to)` slot.
    pub fn covers(&self, time: DateTime<Utc>) -> bool {
        self.valid_from <= time && time < self.valid_to
    }
}

/// Unit rates sorted by `valid_from`, for Agile (half-hourly) or Tracker (daily) tariffs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<Rate>", into = "Vec<Rate>")]
pub struct Rates {
    data: Vec<Rate>,
}

impl From<Vec<Rate>> for Rates {
    fn from(data: Vec<Rate>) -> Self {
        Self::new(data)
    }
}

impl From<Rates> for Vec<Rate> {
    fn from(rates: Rates) -> Self {
        rates.data
    }
}

impl Rates {
    /// Creates a collection, sorting by `valid_from`.
    pub fn new(mut data: Vec<Rate>) -> Self {
        data.sort_by_key(|rate| rate.valid_from);
        Self { data }
    }

    pub fn as_slice(&self) -> &[Rate] {
        &self.data
    }

    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Prices (inc. VAT) in chronological order.
    pub fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.data.iter().map(|rate| rate.value_inc_vat)
    }

    /// Rates starting at or after `start`.
    pub fn starting_from(&self, start: DateTime<Utc>) -> &[Rate] {
        &self.data[self.data.partition_point(|rate| rate.valid_from < start)..]
    }

    /// Rates starting within `[start, end)`.
    pub fn starting_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> &[Rate] {
        let rates = self.starting_from(start);
        &rates[..rates.partition_point(|rate| rate.valid_from < end)]
    }

    /// Rates starting on the given London calendar date.
    pub fn for_date(&self, date: NaiveDate) -> &[Rate] {
        let next_day = date
            .checked_add_days(Days::new(1))
            .expect("date is far from the end of the calendar");
        self.starting_between(london_midnight_utc(date), london_midnight_utc(next_day))
    }

    /// The rate in effect at `time`, or `None` if no slot covers it.
    pub fn rate_at(&self, time: DateTime<Utc>) -> Option<&Rate> {
        let index = self.data.partition_point(|rate| rate.valid_from <= time);
        let rate = self.data.get(index.checked_sub(1)?)?;
        rate.covers(time).then_some(rate)
    }

    /// The rate immediately after the one in effect at `time`.
    pub fn next_rate(&self, time: DateTime<Utc>) -> Option<&Rate> {
        self.rate_at(self.rate_at(time)?.valid_to)
    }

    /// The cheapest rate starting within `[start, end)`; the earliest wins a tie.
    pub fn cheapest_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Option<&Rate> {
        self.starting_between(start, end)
            .iter()
            .min_by(|a, b| a.value_inc_vat.total_cmp(&b.value_inc_vat))
    }

    /// The unbroken run of rates lasting `duration`, starting at or after `from`, with the
    /// lowest average price; the earliest wins a tie.
    pub fn cheapest_window(&self, from: DateTime<Utc>, duration: Duration) -> Option<PriceWindow> {
        let rates = self.starting_from(from);
        (0..rates.len())
            .filter_map(|first| {
                let start = rates[first].valid_from;
                let mut end = start;
                let mut sum = 0.0;
                let mut count = 0_usize;
                for rate in &rates[first..] {
                    if rate.valid_from != end || end - start >= duration {
                        break;
                    }
                    end = rate.valid_to;
                    sum += rate.value_inc_vat;
                    count += 1;
                }
                (end - start == duration).then(|| PriceWindow {
                    start,
                    end,
                    average: sum / count as f64,
                })
            })
            .min_by(|a, b| a.average.total_cmp(&b.average))
    }

    /// Price statistics for the given London calendar date.
    pub fn stats_for_date(&self, date: NaiveDate) -> Option<PriceStats> {
        PriceStats::from_values(self.for_date(date).iter().map(|rate| rate.value_inc_vat))
    }

    /// Price statistics across every rate.
    pub fn stats(&self) -> Option<PriceStats> {
        PriceStats::from_values(self.values())
    }
}

/// A span of consecutive rates and their average price.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub average: f64,
}

/// Range and average of a set of prices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub count: usize,
}

impl PriceStats {
    /// Summarises `values`, or `None` if there are none.
    pub fn from_values(values: impl IntoIterator<Item = f64>) -> Option<Self> {
        let mut stats = Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            avg: 0.0,
            count: 0,
        };
        let mut sum = 0.0;
        for value in values {
            stats.min = stats.min.min(value);
            stats.max = stats.max.max(value);
            sum += value;
            stats.count += 1;
        }
        (stats.count > 0).then(|| Self {
            avg: sum / stats.count as f64,
            ..stats
        })
    }
}

impl fmt::Display for PriceStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}p – {:.2}p", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn rate(valid_from: DateTime<Utc>, minutes: i64, value: f64) -> Rate {
        Rate {
            value_inc_vat: value,
            value_exc_vat: value / 1.05,
            valid_from,
            valid_to: valid_from + Duration::minutes(minutes),
        }
    }

    fn half_hour(h: u32, min: u32, value: f64) -> Rate {
        rate(utc(2024, 1, 15, h, min), 30, value)
    }

    #[test]
    fn new_sorts_by_start_time() {
        let rates = Rates::new(vec![half_hour(11, 0, 2.0), half_hour(10, 0, 1.0)]);
        assert_eq!(rates.values().collect::<Vec<_>>(), vec![1.0, 2.0]);
    }

    #[test]
    fn rate_at_finds_the_covering_slot() {
        let rates = Rates::new(vec![
            half_hour(10, 0, 15.0),
            half_hour(10, 30, 20.0),
            half_hour(11, 0, 25.0),
        ]);

        assert_eq!(
            rates
                .rate_at(utc(2024, 1, 15, 10, 45))
                .unwrap()
                .value_inc_vat,
            20.0
        );
        assert_eq!(
            rates
                .rate_at(utc(2024, 1, 15, 10, 30))
                .unwrap()
                .value_inc_vat,
            20.0
        );
        assert!(rates.rate_at(utc(2024, 1, 15, 9, 59)).is_none());
    }

    #[test]
    fn rate_at_returns_none_in_a_gap() {
        let rates = Rates::new(vec![half_hour(10, 0, 15.0)]);
        assert!(rates.rate_at(utc(2024, 1, 15, 10, 45)).is_none());
    }

    #[test]
    fn next_rate_spans_midnight() {
        let rates = Rates::new(vec![half_hour(23, 0, 10.0), half_hour(23, 30, 20.0)]);
        let next = rates.next_rate(utc(2024, 1, 15, 23, 29)).unwrap();
        assert_eq!(next.value_inc_vat, 20.0);
    }

    #[test]
    fn for_date_respects_london_midnight() {
        let rates = Rates::new(vec![
            half_hour(23, 30, 15.0),
            rate(utc(2024, 1, 16, 0, 0), 30, 20.0),
        ]);

        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        assert_eq!(rates.for_date(today).len(), 1);
        assert_eq!(rates.for_date(today)[0].value_inc_vat, 15.0);
        assert_eq!(rates.for_date(tomorrow)[0].value_inc_vat, 20.0);
    }

    #[test]
    fn for_date_uses_bst_midnight_in_summer() {
        // 23:00 UTC on 1 June is 00:00 BST on 2 June.
        let rates = Rates::new(vec![
            rate(utc(2026, 6, 1, 22, 30), 30, 1.0),
            rate(utc(2026, 6, 1, 23, 0), 30, 2.0),
        ]);

        let june_2 = NaiveDate::from_ymd_opt(2026, 6, 2).unwrap();
        assert_eq!(rates.for_date(june_2).len(), 1);
        assert_eq!(rates.for_date(june_2)[0].value_inc_vat, 2.0);
    }

    #[test]
    fn stats_for_date_summarises_that_day_only() {
        let rates = Rates::new(vec![
            rate(utc(2024, 1, 14, 12, 0), 30, 99.0),
            half_hour(0, 0, 10.0),
            half_hour(12, 0, 20.0),
            half_hour(23, 30, 15.0),
        ]);

        let stats = rates
            .stats_for_date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
            .unwrap();
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 20.0);
        assert_eq!(stats.avg, 15.0);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.to_string(), "10.00p – 20.00p");
    }

    #[test]
    fn stats_for_date_without_data_is_none() {
        let rates = Rates::new(vec![rate(utc(2024, 1, 14, 12, 0), 30, 10.0)]);
        assert!(
            rates
                .stats_for_date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
                .is_none()
        );
    }

    #[test]
    fn cheapest_window_finds_the_lowest_average_run() {
        let rates = Rates::new(vec![
            half_hour(10, 0, 30.0),
            half_hour(10, 30, 5.0),
            half_hour(11, 0, 6.0),
            half_hour(11, 30, 40.0),
            half_hour(12, 0, 1.0),
        ]);

        let window = rates
            .cheapest_window(utc(2024, 1, 15, 10, 0), Duration::hours(1))
            .unwrap();
        assert_eq!(window.start, utc(2024, 1, 15, 10, 30));
        assert_eq!(window.end, utc(2024, 1, 15, 11, 30));
        assert_eq!(window.average, 5.5);
    }

    #[test]
    fn cheapest_window_never_spans_a_gap() {
        // 10:30 is missing, so the two cheap slots are not one hour of continuous supply.
        let rates = Rates::new(vec![
            half_hour(10, 0, 1.0),
            half_hour(11, 0, 1.0),
            half_hour(11, 30, 50.0),
        ]);

        let window = rates
            .cheapest_window(utc(2024, 1, 15, 10, 0), Duration::hours(1))
            .unwrap();
        assert_eq!(window.start, utc(2024, 1, 15, 11, 0));
    }

    #[test]
    fn cheapest_window_needs_enough_published_rates() {
        let rates = Rates::new(vec![half_hour(10, 0, 1.0), half_hour(10, 30, 1.0)]);
        assert!(
            rates
                .cheapest_window(utc(2024, 1, 15, 10, 0), Duration::hours(4))
                .is_none()
        );
    }

    #[test]
    fn cheapest_window_prefers_the_earliest_tie() {
        let rates = Rates::new(vec![half_hour(10, 0, 2.0), half_hour(10, 30, 2.0)]);
        let window = rates
            .cheapest_window(utc(2024, 1, 15, 10, 0), Duration::minutes(30))
            .unwrap();
        assert_eq!(window.start, utc(2024, 1, 15, 10, 0));
    }

    #[test]
    fn cheapest_between_includes_start_and_excludes_end() {
        let rates = Rates::new(vec![
            half_hour(10, 0, 9.0),
            half_hour(10, 30, 5.0),
            half_hour(11, 0, 1.0),
        ]);

        let cheapest = rates
            .cheapest_between(utc(2024, 1, 15, 10, 0), utc(2024, 1, 15, 11, 0))
            .unwrap();
        assert_eq!(cheapest.value_inc_vat, 5.0);
    }

    #[test]
    fn deserializing_restores_sort_order() {
        let json =
            serde_json::to_string(&vec![half_hour(11, 0, 2.0), half_hour(10, 0, 1.0)]).unwrap();
        let rates: Rates = serde_json::from_str(&json).unwrap();
        assert_eq!(rates.values().collect::<Vec<_>>(), vec![1.0, 2.0]);
    }

    #[test]
    fn tracker_style_daily_rates() {
        // Tracker rates run London midnight to midnight (23:00 UTC in summer).
        let rates = Rates::new(vec![
            rate(utc(2026, 6, 1, 23, 0), 24 * 60, 20.67),
            rate(utc(2026, 6, 2, 23, 0), 24 * 60, 22.35),
        ]);
        let now = utc(2026, 6, 2, 23, 30); // 00:30 BST on 3 June

        assert_eq!(rates.rate_at(now).unwrap().value_inc_vat, 22.35);
        let june_3 = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        assert_eq!(rates.for_date(june_3)[0].value_inc_vat, 22.35);
    }
}
