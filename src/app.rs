use yew::prelude::*;

use crate::components::chart::Chart;
use crate::components::status::Status;
use crate::components::summary::Summary;
use crate::components::tracker_display::TrackerDisplay;
use crate::components::{CarbonDisplay, CheapestPeriod, RegionSelector, ThemeToggle, TraceBanner};
use crate::hooks::use_carbon::use_carbon_intensity;
use crate::hooks::use_historical_rates::use_historical_rates;
use crate::hooks::use_now::use_now;
use crate::hooks::use_rates::use_rates;
use crate::hooks::use_region::use_region;
use crate::hooks::use_theme::{ColorScheme, use_theme};
use crate::hooks::use_tracker::use_tracker_rates;

#[function_component(App)]
pub fn app() -> Html {
    let region_handle = use_region();
    let region = region_handle.region;
    let now = use_now();

    let state = use_rates(region).state;
    let historical_state = use_historical_rates(region).state;
    let tracker_state = use_tracker_rates(region).state;
    let carbon_state = use_carbon_intensity().state;
    let theme_handle = use_theme();

    // Extract all historical rate values for banner (31 days × 48 half-hours = ~1488 points)
    let banner_values = use_memo(historical_state.clone(), |state| {
        state
            .data()
            .map_or_else(Vec::new, |rates| rates.values().collect())
    });

    html! {
        <div class="app-container">
            <header class="app-header">
                <CheapestPeriod rates={state.data().cloned()} {now} />
                <h1>{"Octopus Agile Dashboard"}</h1>
                <RegionSelector region={region} on_change={region_handle.set_region.clone()} />
                <ThemeToggle
                    preference={theme_handle.preference}
                    on_cycle={theme_handle.cycle.clone()}
                />
            </header>

            <main class="app-main">
                // Banner section - only show when historical data is loaded and values exist
                if let Some(_rates) = historical_state.data() {
                    if !banner_values.is_empty() {
                        <section class="banner-section">
                            <TraceBanner
                                values={banner_values.clone()}
                                height={100}
                                stroke_width={2.0}
                                smooth={true}
                            />
                        </section>
                    }
                }

                if let Some(rates) = state.data() {
                    <section class="data-section">
                        <h2>{"Agile Electricity"}</h2>
                        <Summary rates={rates.clone()} {now} />
                    </section>

                    {
                        match (tracker_state.data(), tracker_state.error()) {
                            (Some(tracker_rates), _) => html! {
                                <section class="tracker-section">
                                    <h2>{"Tracker Electricity"}</h2>
                                    <TrackerDisplay rates={tracker_rates.clone()} {now} />
                                </section>
                            },
                            (None, Some(err)) => html! {
                                <section class="tracker-section">
                                    <h2>{"Tracker Electricity"}</h2>
                                    <p class="error">{format!("Error loading tracker data: {}", err)}</p>
                                </section>
                            },
                            (None, None) => html! {
                                <section class="tracker-section">
                                    <h2>{"Tracker Electricity"}</h2>
                                    <p>{"Loading tracker data..."}</p>
                                </section>
                            },
                        }
                    }

                    // Chart
                    <section class="chart-section">
                        <h2>{"Energy Price Distribution"}</h2>
                        <Chart rates={rates.clone()} {now} dark_mode={theme_handle.scheme == ColorScheme::Dark} />
                    </section>

                    // Carbon tracking
                    {
                        match (carbon_state.data(), carbon_state.error()) {
                            (Some(carbon_data), _) => html! {
                                <section class="carbon-section">
                                    <h2>{"Grid Carbon Intensity"}</h2>
                                    <CarbonDisplay data={carbon_data.clone()} />
                                </section>
                            },
                            (None, Some(err)) => html! {
                                <section class="carbon-section">
                                    <h2>{"Grid Carbon Intensity"}</h2>
                                    <p class="error">{format!("Error loading carbon data: {}", err)}</p>
                                </section>
                            },
                            (None, None) => html! {
                                <section class="carbon-section">
                                    <h2>{"Grid Carbon Intensity"}</h2>
                                    <p>{"Loading carbon intensity data..."}</p>
                                </section>
                            },
                        }
                    }
                }
            </main>

            <footer class="app-footer">
                <section class="status-section">
                    <h2>{"API Status"}</h2>
                    <Status state={state.clone()} />
                </section>
            </footer>
        </div>
    }
}
