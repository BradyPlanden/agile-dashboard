use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, NaiveTime, Utc, Weekday};

const BST_OFFSET_SECONDS: i32 = 60 * 60;

pub fn london_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&london_offset(dt))
}

pub fn london_date(dt: DateTime<Utc>) -> NaiveDate {
    london_time(dt).date_naive()
}

pub fn london_today() -> NaiveDate {
    london_date(Utc::now())
}

pub fn london_midnight_utc(date: NaiveDate) -> DateTime<Utc> {
    let offset_seconds = london_midnight_offset_seconds(date);
    let utc_midnight =
        date.and_time(NaiveTime::MIN) - chrono::Duration::seconds(i64::from(offset_seconds));
    utc_midnight.and_utc()
}

fn london_offset(dt: DateTime<Utc>) -> FixedOffset {
    let seconds = if is_bst(dt) { BST_OFFSET_SECONDS } else { 0 };
    FixedOffset::east_opt(seconds).expect("London UTC offset is always valid")
}

fn is_bst(dt: DateTime<Utc>) -> bool {
    let year = dt.year();
    let bst_start = last_sunday(year, 3)
        .and_time(NaiveTime::from_hms_opt(1, 0, 0).unwrap())
        .and_utc();
    let bst_end = last_sunday(year, 10)
        .and_time(NaiveTime::from_hms_opt(1, 0, 0).unwrap())
        .and_utc();

    dt >= bst_start && dt < bst_end
}

fn london_midnight_offset_seconds(date: NaiveDate) -> i32 {
    let bst_start_date = last_sunday(date.year(), 3);
    let bst_end_date = last_sunday(date.year(), 10);

    if date > bst_start_date && date <= bst_end_date {
        BST_OFFSET_SECONDS
    } else {
        0
    }
}

fn last_sunday(year: i32, month: u32) -> NaiveDate {
    let mut date = NaiveDate::from_ymd_opt(year, month + 1, 1)
        .expect("valid month")
        .pred_opt()
        .expect("month has a previous day");

    while date.weekday() != Weekday::Sun {
        date = date
            .pred_opt()
            .expect("date can move backwards within month");
    }

    date
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn london_time_switches_to_bst_after_transition() {
        let before = Utc.with_ymd_and_hms(2026, 3, 29, 0, 30, 0).unwrap();
        let after = Utc.with_ymd_and_hms(2026, 3, 29, 1, 30, 0).unwrap();

        assert_eq!(
            london_time(before).format("%Y-%m-%d %H:%M").to_string(),
            "2026-03-29 00:30"
        );
        assert_eq!(
            london_time(after).format("%Y-%m-%d %H:%M").to_string(),
            "2026-03-29 02:30"
        );
    }

    #[test]
    fn london_midnight_utc_handles_spring_forward_day() {
        let start = london_midnight_utc(NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
        let end = london_midnight_utc(NaiveDate::from_ymd_opt(2026, 3, 30).unwrap());

        assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 29, 0, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 3, 29, 23, 0, 0).unwrap());
    }

    #[test]
    fn london_midnight_utc_handles_fall_back_day() {
        let start = london_midnight_utc(NaiveDate::from_ymd_opt(2026, 10, 25).unwrap());
        let end = london_midnight_utc(NaiveDate::from_ymd_opt(2026, 10, 26).unwrap());

        assert_eq!(start, Utc.with_ymd_and_hms(2026, 10, 24, 23, 0, 0).unwrap());
        assert_eq!(end, Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap());
    }
}
