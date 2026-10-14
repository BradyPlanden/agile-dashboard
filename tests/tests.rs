use agile_dashboard::models::rates::{Rate, Rates};
use agile_dashboard::utils::time::{london_date, london_midnight_utc};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

fn utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
}

const fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Contiguous half-hourly rates from London midnight on `first` to London midnight after
/// `last`, priced by slot index.
fn agile_days(first: NaiveDate, last: NaiveDate) -> Rates {
    let end = london_midnight_utc(last.succ_opt().unwrap());
    let mut start = london_midnight_utc(first);
    let mut rates = Vec::new();
    while start < end {
        rates.push(Rate {
            value_inc_vat: rates.len() as f64,
            value_exc_vat: rates.len() as f64 / 1.05,
            valid_from: start,
            valid_to: start + Duration::minutes(30),
        });
        start += Duration::minutes(30);
    }
    Rates::new(rates)
}

#[test]
fn agile_days_have_the_right_number_of_slots_across_clock_changes() {
    let rates = agile_days(date(2026, 3, 28), date(2026, 3, 30));

    assert_eq!(rates.for_date(date(2026, 3, 28)).len(), 48);
    assert_eq!(rates.for_date(date(2026, 3, 29)).len(), 46);
    assert_eq!(rates.for_date(date(2026, 3, 30)).len(), 48);

    let autumn = agile_days(date(2026, 10, 25), date(2026, 10, 25));
    assert_eq!(autumn.for_date(date(2026, 10, 25)).len(), 50);
}

#[test]
fn current_next_and_tomorrow_are_found_late_on_a_bst_evening() {
    // 23:40 UTC on 1 June is 00:40 BST on 2 June: "today" is already the 2nd in London.
    let rates = agile_days(date(2026, 6, 1), date(2026, 6, 3));
    let now = utc(2026, 6, 1, 23, 40);

    assert_eq!(london_date(now), date(2026, 6, 2));
    let current = rates.rate_at(now).unwrap();
    assert_eq!(current.valid_from, utc(2026, 6, 1, 23, 30));
    assert_eq!(
        rates.next_rate(now).unwrap().valid_from,
        utc(2026, 6, 2, 0, 0)
    );
    assert_eq!(rates.stats_for_date(date(2026, 6, 3)).unwrap().count, 48);
}

#[test]
fn cheapest_upcoming_slot_considers_the_current_slot() {
    let mut data: Vec<Rate> = agile_days(date(2026, 1, 15), date(2026, 1, 15))
        .as_slice()
        .to_vec();
    for rate in &mut data {
        rate.value_inc_vat = 20.0;
    }
    data[28].value_inc_vat = 4.0; // 14:00, the current slot
    data[31].value_inc_vat = 3.0; // 15:30
    data[35].value_inc_vat = 1.0; // 17:30, more than three hours away
    let rates = Rates::new(data);
    let now = utc(2026, 1, 15, 14, 10);

    let cheapest = rates
        .cheapest_between(utc(2026, 1, 15, 14, 0), now + Duration::hours(3))
        .unwrap();
    assert_eq!(cheapest.valid_from, utc(2026, 1, 15, 15, 30));
}

#[test]
fn tracker_daily_rates_switch_at_london_midnight() {
    // Tracker publishes one rate per London day.
    let rates = Rates::new(
        [(date(2026, 6, 1), 20.67), (date(2026, 6, 2), 22.35)]
            .into_iter()
            .map(|(day, price)| Rate {
                value_inc_vat: price,
                value_exc_vat: price / 1.05,
                valid_from: london_midnight_utc(day),
                valid_to: london_midnight_utc(day.succ_opt().unwrap()),
            })
            .collect(),
    );

    let evening = utc(2026, 6, 1, 22, 30); // 23:30 BST on 1 June
    let after_midnight = utc(2026, 6, 1, 23, 30); // 00:30 BST on 2 June

    assert_eq!(rates.rate_at(evening).unwrap().value_inc_vat, 20.67);
    assert_eq!(rates.for_date(date(2026, 6, 2))[0].value_inc_vat, 22.35);
    assert_eq!(rates.rate_at(after_midnight).unwrap().value_inc_vat, 22.35);
}

#[test]
fn rates_deserialize_from_the_api_shape() {
    let json = r#"[{
        "value_inc_vat": 15.5,
        "value_exc_vat": 14.76,
        "valid_from": "2025-10-04T00:00:00Z",
        "valid_to": "2025-10-04T00:30:00Z"
    }]"#;

    let rates: Rates = serde_json::from_str(json).unwrap();
    assert_eq!(rates.len(), 1);
    assert_eq!(rates.as_slice()[0].valid_to, utc(2025, 10, 4, 0, 30));
}
