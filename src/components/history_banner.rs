use std::rc::Rc;

use yew::prelude::*;

use crate::config::HISTORY_DAYS;
use crate::models::rates::Rates;
use crate::models::region::Region;
use crate::utils::svg::{Point, fit_to_box, lttb, smooth_path};

// A fixed viewBox stretched by `preserveAspectRatio="none"`, with a non-scaling stroke,
// stays crisp at any width without measuring the container.
const VIEW_WIDTH: f64 = 1000.0;
const VIEW_HEIGHT: f64 = 100.0;
const PADDING: f64 = 4.0;
const MAX_POINTS: usize = 600;

#[derive(Properties, PartialEq)]
pub struct HistoryBannerProps {
    pub rates: Rc<Rates>,
    pub region: Region,
}

/// A sparkline of the last month of Agile prices.
#[function_component(HistoryBanner)]
pub fn history_banner(props: &HistoryBannerProps) -> Html {
    let path = use_memo(props.rates.clone(), |rates| {
        let samples: Vec<Point> = rates
            .values()
            .enumerate()
            .map(|(i, value)| (i as f64, value))
            .collect();
        smooth_path(&fit_to_box(
            &lttb(&samples, MAX_POINTS),
            VIEW_WIDTH,
            VIEW_HEIGHT,
            PADDING,
        ))
    });

    html! {
        <svg
            class="history-trace"
            viewBox={format!("0 0 {VIEW_WIDTH} {VIEW_HEIGHT}")}
            preserveAspectRatio="none"
            role="img"
            aria-label={describe(&props.rates, props.region)}
        >
            <path d={(*path).clone()} />
        </svg>
    }
}

/// Screen-reader description of the trace, which has no visible caption.
fn describe(rates: &Rates, region: Region) -> String {
    let summary = rates.stats().map_or_else(String::new, |stats| {
        format!(
            ": low {:.2}p, average {:.2}p, high {:.2}p",
            stats.min, stats.avg, stats.max
        )
    });
    format!("Agile prices over the last {HISTORY_DAYS} days in {region}{summary}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rates::Rate;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn describes_region_and_price_range() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let rates = Rates::new(
            (0_i64..)
                .zip([10.0, -2.0, 30.0])
                .map(|(i, price)| {
                    let valid_from = start + Duration::minutes(30 * i);
                    Rate {
                        value_inc_vat: price,
                        value_exc_vat: price / 1.05,
                        valid_from,
                        valid_to: valid_from + Duration::minutes(30),
                    }
                })
                .collect(),
        );

        assert_eq!(
            describe(&rates, Region::C),
            "Agile prices over the last 31 days in London (C): low -2.00p, average 12.67p, high 30.00p"
        );
        assert_eq!(
            describe(&Rates::default(), Region::M),
            "Agile prices over the last 31 days in Yorkshire (M)"
        );
    }
}
