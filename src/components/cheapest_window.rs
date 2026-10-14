use std::rc::Rc;

use chrono::{DateTime, Duration, Utc};
use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;

use crate::config::{CHEAPEST_WINDOW_DEFAULT_HOURS, CHEAPEST_WINDOW_MAX_HOURS};
use crate::models::rates::Rates;
use crate::utils::log;
use crate::utils::time::{london_hhmm, slot_start, start_label};

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

    let window = props
        .rates
        .cheapest_window(slot_start(props.now), Duration::hours(i64::from(*hours)));
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
                    { start_label(window.start, props.now) }
                    { format!("–{}", london_hhmm(window.end)) }
                </dd>
                <dd class="stat-sub">{ format!("average {:.2}p", window.average) }</dd>
            } else {
                <dd class="stat-value" aria-live="polite">{ "N/A" }</dd>
                <dd class="stat-sub">{ "Not enough prices published" }</dd>
            }
        </div>
    }
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
