use std::rc::Rc;

use chrono::{DateTime, Duration, Utc};
use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;

use crate::config::{CHEAPEST_WINDOW_DEFAULT_HOURS, CHEAPEST_WINDOW_MAX_HOURS};
use crate::models::rates::{PriceWindow, Rates};
use crate::utils::log;
use crate::utils::time::london_hhmm;

const STORAGE_KEY: &str = "cheapest_window_hours";

#[derive(Properties, PartialEq)]
pub struct CheapestWindowProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

/// The cheapest upcoming block of prices, with a stepper to choose its length in hours.
#[function_component(CheapestWindow)]
pub fn cheapest_window(props: &CheapestWindowProps) -> Html {
    let hours = use_state(load_hours);

    use_effect_with(*hours, |hours| {
        if let Err(error) = LocalStorage::set(STORAGE_KEY, hours) {
            log::warn(&format!("Failed to save window length: {error}"));
        }
    });

    let window = upcoming_window(&props.rates, props.now, *hours);
    let step = |delta: i8| {
        let hours = hours.clone();
        Callback::from(move |_: MouseEvent| hours.set(step_hours(*hours, delta)))
    };

    html! {
        <div class="stat">
            <dt class="window-label">
                { "Cheapest" }
                <span class="stepper">
                    <button
                        type="button"
                        class="stepper-button"
                        aria-label="Shorter window"
                        disabled={*hours <= 1}
                        onclick={step(-1)}
                    >
                        { "−" }
                    </button>
                    <span class="stepper-value">
                        { format!("{} hour{}", *hours, if *hours == 1 { "" } else { "s" }) }
                    </span>
                    <button
                        type="button"
                        class="stepper-button"
                        aria-label="Longer window"
                        disabled={*hours >= CHEAPEST_WINDOW_MAX_HOURS}
                        onclick={step(1)}
                    >
                        { "+" }
                    </button>
                </span>
            </dt>
            if let Some(window) = window {
                <dd class="stat-value" aria-live="polite">
                    { format!("{}–{}", london_hhmm(window.start), london_hhmm(window.end)) }
                </dd>
                <dd class="stat-sub">{ format!("average {:.2}p", window.average) }</dd>
            } else {
                <dd class="stat-value" aria-live="polite">{ "N/A" }</dd>
                <dd class="stat-sub">{ "Not enough prices published" }</dd>
            }
        </div>
    }
}

/// The cheapest block of `hours` that is still wholly ahead of `now`.
///
/// It starts no earlier than the next half hour, so it always lasts the full length and its
/// average leaves out the part of the current slot that has already gone.
fn upcoming_window(rates: &Rates, now: DateTime<Utc>, hours: u8) -> Option<PriceWindow> {
    rates.cheapest_window(now, Duration::hours(i64::from(hours)))
}

fn load_hours() -> u8 {
    LocalStorage::get(STORAGE_KEY)
        .ok()
        .filter(|hours| (1..=CHEAPEST_WINDOW_MAX_HOURS).contains(hours))
        .unwrap_or(CHEAPEST_WINDOW_DEFAULT_HOURS)
}

fn step_hours(hours: u8, delta: i8) -> u8 {
    hours
        .saturating_add_signed(delta)
        .clamp(1, CHEAPEST_WINDOW_MAX_HOURS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rates::Rate;
    use chrono::TimeZone;

    fn utc(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 14, hour, minute, 0).unwrap()
    }

    /// Half-hourly rates from 14:00, priced in order.
    fn rates_from_two_pm(prices: &[f64]) -> Rates {
        let mut valid_from = utc(14, 0);
        let rates = prices.iter().map(|&price| {
            let rate = Rate {
                value_inc_vat: price,
                value_exc_vat: price / 1.05,
                valid_from,
                valid_to: valid_from + Duration::minutes(30),
            };
            valid_from = rate.valid_to;
            rate
        });
        Rates::new(rates.collect())
    }

    #[test]
    fn window_skips_the_slot_already_under_way() {
        // 14:00 is by far the cheapest slot, but at 14:29 only a minute of it is left.
        let rates = rates_from_two_pm(&[1.0, 20.0, 20.0, 30.0]);
        let window = upcoming_window(&rates, utc(14, 29), 1).unwrap();

        assert_eq!((window.start, window.end), (utc(14, 30), utc(15, 30)));
        assert_eq!(window.end - window.start, Duration::hours(1));
        assert_eq!(window.average, 20.0);
    }

    #[test]
    fn window_can_start_at_a_slot_that_begins_now() {
        let rates = rates_from_two_pm(&[1.0, 20.0, 20.0, 30.0]);
        let window = upcoming_window(&rates, utc(14, 0), 1).unwrap();

        assert_eq!((window.start, window.end), (utc(14, 0), utc(15, 0)));
    }

    #[test]
    fn stepping_stays_within_bounds() {
        assert_eq!(step_hours(4, 1), 5);
        assert_eq!(step_hours(4, -1), 3);
        assert_eq!(step_hours(1, -1), 1);
        assert_eq!(
            step_hours(CHEAPEST_WINDOW_MAX_HOURS, 1),
            CHEAPEST_WINDOW_MAX_HOURS
        );
    }
}
