//! Octopus Energy product API: Agile and Tracker unit rates.

use chrono::{DateTime, Days, NaiveDate, Utc};
use serde::Deserialize;

use crate::config::HISTORY_DAYS;
use crate::models::error::AppError;
use crate::models::rates::{Rate, Rates};
use crate::models::region::Region;
use crate::services::http;
use crate::utils::log;
use crate::utils::time::{london_date, london_midnight_utc};

const BASE_URL: &str = "https://api.octopus.energy/v1/products";
const TRUSTED_ORIGIN: &str = "https://api.octopus.energy/";
const AGILE_PRODUCT: &str = "AGILE-24-10-01";
const TRACKER_PRODUCT: &str = "SILVER-24-10-01";
const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

/// The API maximum. 31 days of half-hourly slots (≤ 1,490) fits in one page, where the
/// default of 100 took 15 sequential requests.
const PAGE_SIZE: u32 = 1500;

#[derive(Deserialize)]
struct Page {
    results: Vec<Rate>,
    #[serde(default)]
    next: Option<String>,
}

/// Agile rates for today and tomorrow (London time).
pub async fn fetch_agile_rates(region: Region) -> Result<Rates, AppError> {
    let (from, to) = current_window(Utc::now());
    fetch_unit_rates(unit_rates_url(AGILE_PRODUCT, region, from, to)).await
}

/// Tracker rates for today and tomorrow (London time).
pub async fn fetch_tracker_rates(region: Region) -> Result<Rates, AppError> {
    let (from, to) = current_window(Utc::now());
    fetch_unit_rates(unit_rates_url(TRACKER_PRODUCT, region, from, to)).await
}

/// Agile rates for the `HISTORY_DAYS` whole days before `today` (London time).
pub async fn fetch_agile_history(region: Region, today: NaiveDate) -> Result<Rates, AppError> {
    let (from, to) = history_window(today);
    fetch_unit_rates(unit_rates_url(AGILE_PRODUCT, region, from, to)).await
}

/// From London midnight today to London midnight the day after tomorrow.
fn current_window(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let today = london_date(now);
    let end = today
        .checked_add_days(Days::new(2))
        .expect("two days after today is a valid date");
    (london_midnight_utc(today), london_midnight_utc(end))
}

/// From London midnight `HISTORY_DAYS` before `today` to London midnight starting `today`.
pub fn history_window(today: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = today
        .checked_sub_days(Days::new(HISTORY_DAYS))
        .expect("history start is a valid date");
    (london_midnight_utc(start), london_midnight_utc(today))
}

fn unit_rates_url(product: &str, region: Region, from: DateTime<Utc>, to: DateTime<Utc>) -> String {
    format!(
        "{BASE_URL}/{product}/electricity-tariffs/E-1R-{product}-{code}/standard-unit-rates/\
         ?period_from={from}&period_to={to}&page_size={PAGE_SIZE}",
        code = region.code(),
        from = from.format(TIMESTAMP_FORMAT),
        to = to.format(TIMESTAMP_FORMAT),
    )
}

async fn fetch_unit_rates(url: String) -> Result<Rates, AppError> {
    let mut rates = Vec::new();
    let mut next = Some(url);
    while let Some(url) = next {
        let page: Page = http::get_json(&url).await?;
        rates.extend(page.results);
        next = next_page(page.next)?;
    }
    Ok(Rates::new(rates))
}

/// The next page to fetch. A link that leaves the Octopus API is an error rather than the
/// end of the data, so the pages already fetched are never mistaken for a complete result.
fn next_page(next: Option<String>) -> Result<Option<String>, AppError> {
    match next {
        Some(url) if !is_trusted_page_url(&url) => {
            log::warn(&format!("Refusing pagination link to untrusted URL: {url}"));
            Err(AppError::Parse(
                "the next page of prices is outside the Octopus API".to_owned(),
            ))
        }
        next => Ok(next),
    }
}

/// Only follow `next` links back to the Octopus API, never to an arbitrary host.
fn is_trusted_page_url(url: &str) -> bool {
    url.starts_with(TRUSTED_ORIGIN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    #[test]
    fn unit_rates_url_targets_region_and_requests_one_large_page() {
        let from = Utc.with_ymd_and_hms(2026, 3, 29, 0, 0, 0).unwrap();
        let to = Utc.with_ymd_and_hms(2026, 3, 30, 23, 0, 0).unwrap();

        let url = unit_rates_url(AGILE_PRODUCT, Region::M, from, to);

        assert_eq!(
            url,
            "https://api.octopus.energy/v1/products/AGILE-24-10-01/electricity-tariffs/\
             E-1R-AGILE-24-10-01-M/standard-unit-rates/\
             ?period_from=2026-03-29T00:00:00Z&period_to=2026-03-30T23:00:00Z&page_size=1500"
        );
    }

    #[test]
    fn current_window_uses_london_midnight_in_bst() {
        let now = Utc.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).unwrap();
        let (start, end) = current_window(now);

        assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 29, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 3, 30, 23, 0, 0).unwrap());
    }

    #[test]
    fn history_window_covers_whole_days_before_today_across_a_clock_change() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 12).unwrap();
        let (start, end) = history_window(today);

        // Starts in GMT, ends in BST.
        assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 12, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 4, 11, 23, 0, 0).unwrap());
    }

    #[test]
    fn history_window_moves_at_london_midnight() {
        // 00:00 BST on 12 April is 23:00 UTC on the 11th; `use_now` ticks 250 ms after it.
        let midnight = Utc.with_ymd_and_hms(2026, 4, 11, 23, 0, 0).unwrap();
        let (_, before) = history_window(london_date(midnight - Duration::seconds(1)));
        let (_, after) = history_window(london_date(midnight + Duration::milliseconds(250)));

        assert_eq!(before, Utc.with_ymd_and_hms(2026, 4, 10, 23, 0, 0).unwrap());
        assert_eq!(after, midnight);
    }

    #[test]
    fn page_parses_api_rates() {
        let json = r#"{
            "count": 2,
            "next": null,
            "results": [
                {"value_exc_vat": 10.5, "value_inc_vat": 11.025,
                 "valid_from": "2024-01-01T00:00:00Z", "valid_to": "2024-01-01T00:30:00Z"},
                {"value_exc_vat": 12.0, "value_inc_vat": 12.6,
                 "valid_from": "2024-01-01T00:30:00Z", "valid_to": "2024-01-01T01:00:00Z"}
            ]
        }"#;

        let page: Page = serde_json::from_str(json).unwrap();
        assert_eq!(page.next, None);
        assert_eq!(page.results.len(), 2);
        assert_eq!(page.results[1].value_inc_vat, 12.6);
    }

    #[test]
    fn page_without_next_field_parses() {
        let page: Page = serde_json::from_str(r#"{"results": []}"#).unwrap();
        assert!(page.next.is_none());
    }

    #[test]
    fn last_page_ends_pagination() {
        assert_eq!(next_page(None), Ok(None));
    }

    #[test]
    fn trusted_next_link_is_followed() {
        let url = "https://api.octopus.energy/v1/products/AGILE-24-10-01/?page=2".to_owned();
        assert_eq!(next_page(Some(url.clone())), Ok(Some(url)));
    }

    #[test]
    fn untrusted_next_link_fails_the_fetch() {
        let result = next_page(Some("https://evil.example.com/?page=2".to_owned()));
        assert!(matches!(result, Err(AppError::Parse(_))));
    }

    #[test]
    fn trusts_only_https_octopus_api_links() {
        assert!(is_trusted_page_url(
            "https://api.octopus.energy/v1/products/AGILE-24-10-01/standard-unit-rates/?page=2"
        ));
        assert!(!is_trusted_page_url("https://evil.example.com/api/data"));
        assert!(!is_trusted_page_url(
            "http://api.octopus.energy/v1/products/?page=2"
        ));
        assert!(!is_trusted_page_url("https://api.octopus.energy.evil.com/"));
        assert!(!is_trusted_page_url("https://api.octopus.energy@evil.com/"));
    }
}
