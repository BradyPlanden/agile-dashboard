use chrono::{DateTime, Duration, Utc};
use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::utils::time::slot_start;

/// The current time, re-rendering the caller at each half-hour boundary when Agile prices change.
#[hook]
pub fn use_now() -> DateTime<Utc> {
    let now = use_state(Utc::now);
    {
        let setter = now.setter();
        use_effect_with(*now, move |now| {
            let timeout = Timeout::new(ms_until_next_slot(*now), move || setter.set(Utc::now()));
            move || drop(timeout)
        });
    }
    *now
}

/// Milliseconds from `now` until just after the next :00 or :30.
fn ms_until_next_slot(now: DateTime<Utc>) -> u32 {
    // Land slightly inside the new slot so `Utc::now()` can't read the old one.
    const MARGIN_MS: i64 = 250;
    let next = slot_start(now) + Duration::minutes(30);
    u32::try_from((next - now).num_milliseconds() + MARGIN_MS).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn waits_until_just_after_the_next_half_hour() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 14, 29, 0).unwrap();
        assert_eq!(ms_until_next_slot(now), 60_250);

        let on_boundary = Utc.with_ymd_and_hms(2026, 1, 1, 14, 30, 0).unwrap();
        assert_eq!(ms_until_next_slot(on_boundary), 30 * 60_000 + 250);
    }
}
