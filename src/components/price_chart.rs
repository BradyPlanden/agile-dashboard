use std::rc::Rc;

use chrono::{DateTime, Timelike, Utc};
use yew::prelude::*;

use crate::models::price_band::PriceBand;
use crate::models::rates::{PriceStats, Rate, Rates};
use crate::utils::time::{london_date, london_hhmm, london_midnight_utc, london_time};

#[derive(Properties, PartialEq)]
pub struct PriceChartProps {
    pub rates: Rc<Rates>,
    pub now: DateTime<Utc>,
}

/// Half-hourly Agile prices from the start of today, as colour-banded bars.
///
/// Geometry is drawn in an SVG stretched to fit (`preserveAspectRatio="none"`), with x in
/// slot units and y in percent; labels are HTML positioned by percentage so text stays crisp.
#[function_component(PriceChart)]
pub fn price_chart(props: &PriceChartProps) -> Html {
    let selection = use_reducer_eq(Selection::default);

    let PriceChartProps { rates, now } = props;
    let slots = rates.starting_from(london_midnight_utc(london_date(*now)));
    if slots.is_empty() {
        return html! { <p class="notice">{ "No prices published for today yet." }</p> };
    }

    let count = slots.len();
    let current = slots.iter().position(|rate| rate.covers(*now));
    let selected_index = selection.shown(count);
    let focused = selected_index.or(current);
    let scale = YScale::for_values(slots.iter().map(|rate| rate.value_inc_vat));
    let width = count as f64;

    let bars = slots.iter().enumerate().map(|(index, rate)| {
        let value = rate.value_inc_vat;
        let top = scale.y(value.max(0.0));
        let bottom = scale.y(value.min(0.0));
        let class = classes!(
            "bar",
            PriceBand::for_price(value).css_class(),
            (rate.valid_to <= *now).then_some("past"),
            (Some(index) == selected_index).then_some("selected"),
        );
        html! {
            <rect
                {class}
                x={format!("{:.2}", index as f64 + 0.15)}
                y={format!("{top:.3}")}
                width="0.7"
                height={format!("{:.3}", bottom - top)}
            />
        }
    });

    let hit_targets = (0..count).map(|index| {
        let hover = selection.dispatcher();
        let onpointerenter = Callback::from(move |event: PointerEvent| {
            // Touch has no hover, and the `pointerleave` that follows a lift would clear one at
            // once; the tap pins the bar instead.
            if event.pointer_type() != "touch" {
                hover.dispatch(SelectionAction::Hover(Some(index)));
            }
        });
        let tap = selection.dispatcher();
        let onclick =
            Callback::from(move |_: MouseEvent| tap.dispatch(SelectionAction::Toggle(index)));
        html! {
            <rect class="hit" x={index.to_string()} y="0" width="1" height="100" {onpointerenter} {onclick} />
        }
    });

    let grid_lines = scale.ticks().map(|tick| {
        let y = format!("{:.3}", scale.y(tick));
        let class = classes!("grid", (tick == 0.0).then_some("zero"));
        html! { <line {class} x1="0" x2={width.to_string()} y1={y.clone()} y2={y} /> }
    });

    let day_dividers = slots
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| london_date(pair[0].valid_from) != london_date(pair[1].valid_from))
        .map(|(index, _)| {
            let x = (index + 1).to_string();
            html! { <line class="day-divider" x1={x.clone()} x2={x} y1="0" y2="100" /> }
        });

    let y_labels = scale.ticks().map(|tick| {
        html! {
            <span style={format!("top: {:.2}%", scale.y(tick))}>{ scale.label(tick) }</span>
        }
    });

    let x_labels = slots.iter().enumerate().filter_map(|(index, rate)| {
        let local = london_time(rate.valid_from);
        (local.minute() == 0 && local.hour().is_multiple_of(3)).then(|| {
            let (text, class) = match local.hour() {
                0 => (local.format("%a").to_string(), "day"),
                hour if hour.is_multiple_of(6) => (local.format("%H:%M").to_string(), "major"),
                _ => (local.format("%H:%M").to_string(), "minor"),
            };
            html! {
                <span {class} style={format!("left: {:.2}%", index as f64 / width * 100.0)}>{ text }</span>
            }
        })
    });

    let onpointerleave = {
        let dispatch = selection.dispatcher();
        Callback::from(move |_: PointerEvent| dispatch.dispatch(SelectionAction::Hover(None)))
    };
    let onkeydown = {
        let dispatch = selection.dispatcher();
        Callback::from(move |event: KeyboardEvent| {
            let last = count - 1;
            let target = match event.key().as_str() {
                "ArrowLeft" => Some(focused.map_or(0, |index| index.saturating_sub(1))),
                "ArrowRight" => Some(focused.map_or(0, |index| (index + 1).min(last))),
                "Home" => Some(0),
                "End" => Some(last),
                "Escape" => None,
                _ => return,
            };
            event.prevent_default();
            dispatch.dispatch(SelectionAction::Pin(target));
        })
    };

    let value_text = focused.map_or_else(
        || "No slot selected".to_owned(),
        |index| {
            let rate = &slots[index];
            format!("{}, {:.2}p per kWh", slot_label(rate), rate.value_inc_vat)
        },
    );

    html! {
        <figure class="price-chart">
            <Readout rate={focused.map(|index| slots[index].clone())} is_now={focused.is_some() && focused == current} />
            <div class="chart-body">
                <div class="chart-y" aria-hidden="true">{ for y_labels }</div>
                <div
                    class="chart-plot"
                    tabindex="0"
                    role="slider"
                    aria-label="Half-hour slot"
                    aria-valuemin="0"
                    aria-valuemax={(count - 1).to_string()}
                    aria-valuenow={focused.unwrap_or(0).to_string()}
                    aria-valuetext={value_text}
                    {onpointerleave}
                    {onkeydown}
                >
                    <svg viewBox={format!("0 0 {width} 100")} preserveAspectRatio="none" aria-hidden="true">
                        { for grid_lines }
                        { for day_dividers }
                        { for bars }
                        { for hit_targets }
                    </svg>
                </div>
                <div class="chart-x" aria-hidden="true">{ for x_labels }</div>
            </div>
            <Legend />
            <figcaption class="sr-only">{ describe(slots) }</figcaption>
        </figure>
    }
}

/// The bars picked out by pointer or keyboard. A hovered bar shows over a pinned one, and
/// the chart falls back to the current slot when neither is set.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Selection {
    /// The bar under a mouse or pen. Touch never hovers.
    hovered: Option<usize>,
    /// The bar last tapped, clicked or chosen with the keyboard. It outlasts the pointer
    /// leaving the chart, which is what lets a tap stick.
    pinned: Option<usize>,
}

enum SelectionAction {
    Hover(Option<usize>),
    /// Pins a bar, or releases it if it's already pinned.
    Toggle(usize),
    /// Pins a bar, or releases the pin, and drops any hover so the keyboard choice shows.
    Pin(Option<usize>),
}

impl Selection {
    /// The selected bar among `count`, if any.
    fn shown(self, count: usize) -> Option<usize> {
        self.hovered.or(self.pinned).filter(|&index| index < count)
    }

    fn apply(self, action: SelectionAction) -> Self {
        match action {
            SelectionAction::Hover(hovered) => Self { hovered, ..self },
            SelectionAction::Toggle(index) => Self {
                pinned: (self.pinned != Some(index)).then_some(index),
                ..self
            },
            SelectionAction::Pin(pinned) => Self {
                hovered: None,
                pinned,
            },
        }
    }
}

impl Reducible for Selection {
    type Action = SelectionAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Rc::new(self.apply(action))
    }
}

#[derive(Properties, PartialEq)]
struct ReadoutProps {
    rate: Option<Rate>,
    is_now: bool,
}

/// The price of the hovered, pinned or current slot.
#[function_component(Readout)]
fn readout(props: &ReadoutProps) -> Html {
    html! {
        <p class="chart-readout" aria-live="polite">
            if let Some(rate) = &props.rate {
                <span class={classes!("swatch", PriceBand::for_price(rate.value_inc_vat).css_class())} aria-hidden="true" />
                <span>{ if props.is_now { format!("Now · {}", slot_label(rate)) } else { slot_label(rate) } }</span>
                { " " }
                <strong>{ format!("{:.2}p/kWh", rate.value_inc_vat) }</strong>
            } else {
                { "Hover or tap a bar to see its price." }
            }
        </p>
    }
}

#[function_component(Legend)]
fn legend() -> Html {
    html! {
        <ul class="chart-legend" aria-label="Price bands">
            { for PriceBand::ALL.iter().map(|band| html! {
                <li class={band.css_class()}>
                    <span class="swatch" aria-hidden="true" />
                    { band.label() }
                </li>
            }) }
        </ul>
    }
}

/// e.g. `Wed 14:00–14:30`.
fn slot_label(rate: &Rate) -> String {
    format!(
        "{} {}–{}",
        london_time(rate.valid_from).format("%a"),
        london_hhmm(rate.valid_from),
        london_hhmm(rate.valid_to)
    )
}

/// Screen-reader summary of the chart.
fn describe(slots: &[Rate]) -> String {
    let (Some(first), Some(last)) = (slots.first(), slots.last()) else {
        return String::new();
    };
    let cheapest = slots
        .iter()
        .min_by(|a, b| a.value_inc_vat.total_cmp(&b.value_inc_vat))
        .unwrap_or(first);
    let dearest = slots
        .iter()
        .max_by(|a, b| a.value_inc_vat.total_cmp(&b.value_inc_vat))
        .unwrap_or(first);
    let average = PriceStats::from_values(slots.iter().map(|rate| rate.value_inc_vat))
        .map_or(0.0, |stats| stats.avg);
    format!(
        "Half-hourly prices from {} {} to {} {}: lowest {:.2}p at {}, highest {:.2}p at {}, average {:.2}p.",
        london_time(first.valid_from).format("%A"),
        london_hhmm(first.valid_from),
        london_time(last.valid_to).format("%A"),
        london_hhmm(last.valid_to),
        cheapest.value_inc_vat,
        london_hhmm(cheapest.valid_from),
        dearest.value_inc_vat,
        london_hhmm(dearest.valid_from),
        average,
    )
}

/// A y-axis that always includes zero, with round tick steps.
#[derive(Debug, PartialEq)]
struct YScale {
    min: f64,
    max: f64,
    step: f64,
}

impl YScale {
    const TARGET_TICKS: f64 = 5.0;

    fn for_values(values: impl IntoIterator<Item = f64>) -> Self {
        let (low, high) = values
            .into_iter()
            .fold((0.0_f64, 0.0_f64), |(low, high), value| {
                (low.min(value), high.max(value))
            });
        let step = nice_step((high - low).max(1.0) / Self::TARGET_TICKS);
        let min = (low / step).floor() * step;
        let max = ((high / step).ceil() * step).max(min + step);
        Self { min, max, step }
    }

    /// Vertical position of `value`, in percent from the top.
    fn y(&self, value: f64) -> f64 {
        (self.max - value) / (self.max - self.min) * 100.0
    }

    fn ticks(&self) -> impl Iterator<Item = f64> + '_ {
        (0_u32..)
            .map(|k| f64::from(k).mul_add(self.step, self.min))
            .take_while(|tick| *tick <= self.max + self.step / 2.0)
    }

    fn label(&self, tick: f64) -> String {
        let precision = usize::from(self.step.fract() != 0.0);
        format!("{:.*}p", precision, tick + 0.0)
    }
}

/// The smallest of 1, 2, 2.5, 5 or 10 × a power of ten that is at least `raw`.
fn nice_step(raw: f64) -> f64 {
    let magnitude = 10_f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let nice = [1.0, 2.0, 2.5, 5.0]
        .into_iter()
        .find(|&candidate| normalized <= candidate)
        .unwrap_or(10.0);
    nice * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Duration, TimeZone};

    fn slot(hour: u32, minute: u32, price: f64) -> Rate {
        let valid_from = Utc.with_ymd_and_hms(2026, 1, 14, hour, minute, 0).unwrap();
        Rate {
            value_inc_vat: price,
            value_exc_vat: price / 1.05,
            valid_from,
            valid_to: valid_from + Duration::minutes(30),
        }
    }

    #[test]
    fn slot_label_names_the_day_and_half_hour() {
        assert_eq!(slot_label(&slot(14, 0, 1.0)), "Wed 14:00–14:30");
    }

    #[test]
    fn describe_summarises_range_and_extremes() {
        let slots = [slot(0, 0, 20.0), slot(0, 30, 5.0), slot(1, 0, 35.0)];
        assert_eq!(
            describe(&slots),
            "Half-hourly prices from Wednesday 00:00 to Wednesday 01:30: lowest 5.00p at 00:30, \
             highest 35.00p at 01:00, average 20.00p."
        );
        assert_eq!(describe(&[]), "");
    }

    fn select(actions: impl IntoIterator<Item = SelectionAction>) -> Selection {
        actions
            .into_iter()
            .fold(Selection::default(), Selection::apply)
    }

    #[test]
    fn tapped_bar_stays_selected_after_the_pointer_leaves() {
        // A touch tap: a click, then `pointerleave` once the finger lifts.
        let selection = select([SelectionAction::Toggle(5), SelectionAction::Hover(None)]);
        assert_eq!(selection.shown(48), Some(5));
    }

    #[test]
    fn hover_shows_over_the_pin_until_the_pointer_leaves() {
        let hovering = select([SelectionAction::Toggle(5), SelectionAction::Hover(Some(9))]);
        assert_eq!(hovering.shown(48), Some(9));
        assert_eq!(
            hovering.apply(SelectionAction::Hover(None)).shown(48),
            Some(5)
        );
    }

    #[test]
    fn tapping_the_pinned_bar_releases_it() {
        let selection = select([SelectionAction::Toggle(5), SelectionAction::Toggle(5)]);
        assert_eq!(selection.shown(48), None);
        assert_eq!(
            selection.apply(SelectionAction::Toggle(6)).shown(48),
            Some(6)
        );
    }

    #[test]
    fn keyboard_pin_replaces_hover() {
        let selection = select([
            SelectionAction::Hover(Some(9)),
            SelectionAction::Pin(Some(2)),
        ]);
        assert_eq!(
            selection,
            Selection {
                hovered: None,
                pinned: Some(2)
            }
        );
        assert_eq!(selection.apply(SelectionAction::Pin(None)).shown(48), None);
    }

    #[test]
    fn selection_beyond_the_published_slots_is_ignored() {
        assert_eq!(select([SelectionAction::Toggle(60)]).shown(48), None);
    }

    #[test]
    fn nice_steps_are_round() {
        assert_eq!(nice_step(7.0), 10.0);
        assert_eq!(nice_step(4.2), 5.0);
        assert_eq!(nice_step(2.2), 2.5);
        assert_eq!(nice_step(1.0), 1.0);
        assert_eq!(nice_step(0.3), 0.5);
    }

    #[test]
    fn scale_starts_at_zero_for_positive_prices() {
        let scale = YScale::for_values([6.2, 18.0, 34.5]);
        assert_eq!(scale.min, 0.0);
        assert_eq!(scale.max, 40.0);
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![0.0, 10.0, 20.0, 30.0, 40.0]
        );
        assert_eq!(scale.y(40.0), 0.0);
        assert_eq!(scale.y(0.0), 100.0);
    }

    #[test]
    fn scale_extends_below_zero_for_negative_prices() {
        let scale = YScale::for_values([-3.0, 12.0]);
        assert!(scale.min < 0.0);
        assert!(scale.y(0.0) < 100.0);
        assert!(scale.ticks().any(|tick| tick == 0.0));
    }

    #[test]
    fn labels_show_decimals_only_for_fractional_steps() {
        let whole = YScale {
            min: 0.0,
            max: 20.0,
            step: 5.0,
        };
        let fractional = YScale {
            min: 0.0,
            max: 5.0,
            step: 2.5,
        };
        assert_eq!(whole.label(15.0), "15p");
        assert_eq!(fractional.label(2.5), "2.5p");
        assert_eq!(whole.label(-0.0), "0p");
    }
}
