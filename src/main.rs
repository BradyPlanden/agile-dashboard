use yew::prelude::*;

mod components;
mod config;
mod hooks;
mod models;
mod services;
mod utils;

use components::chart::Chart;
use components::status::Status;
use components::summary::Summary;
use components::tracker_display::TrackerDisplay;
use components::{
    CarbonDisplay, CheapestPeriod, RegionSelector, ThemeToggle, TraceBanner,
};
use hooks::use_carbon::{CarbonDataState, use_carbon_intensity};
use hooks::use_historical_rates::use_historical_rates;
use hooks::use_rates::use_rates;
use hooks::use_region::use_region;
use hooks::use_theme::{Theme, use_theme};
use hooks::use_tracker::use_tracker_rates;

#[function_component(App)]
fn app() -> Html {
    let region_handle = use_region();
    let region = region_handle.region;

    let state = use_rates(region);
    let historical_state = use_historical_rates();
    let tracker_state = use_tracker_rates(region);
    let carbon_state = use_carbon_intensity();
    let theme_handle = use_theme();

    // Extract all historical rate values for banner (31 days × 48 half-hours = ~1488 points)
    let banner_values = use_memo(historical_state.clone(), |state| {
        match state.data() {
            Some(rates) => rates.all_values(),
            None => vec![], // Empty during Loading/Error
        }
    });

    html! {
        <div class="app-container">
            <header class="app-header">
                <CheapestPeriod />
                <h1>{"Octopus Agile Dashboard"}</h1>
                <RegionSelector region={region} on_change={region_handle.set_region.clone()} />
                <ThemeToggle />
            </header>

            <main class="app-main">
                // Banner section - only show when historical data is loaded and values exist
                if let Some(_rates) = historical_state.data() {
                    if !banner_values.is_empty() {
                        <section class="banner-section">
                            <TraceBanner
                                values={(*banner_values).clone()}
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
                        <Summary rates={rates.clone()} />
                    </section>

                    {
                        match &*tracker_state {
                            hooks::use_tracker::TrackerDataState::Loading => html! {
                                <section class="tracker-section">
                                    <h2>{"Tracker Electricity"}</h2>
                                    <p>{"Loading tracker data..."}</p>
                                </section>
                            },
                            hooks::use_tracker::TrackerDataState::Loaded(tracker_rates) => html! {
                                <section class="tracker-section">
                                    <h2>{"Tracker Electricity"}</h2>
                                    <TrackerDisplay rates={tracker_rates.clone()} />
                                </section>
                            },
                            hooks::use_tracker::TrackerDataState::Error(err) => html! {
                                <section class="tracker-section">
                                    <h2>{"Tracker Electricity"}</h2>
                                    <p class="error">{format!("Error loading tracker data: {}", err)}</p>
                                </section>
                            },
                        }
                    }

                    // Chart
                    <section class="chart-section">
                        <h2>{"Energy Price Distribution"}</h2>
                        <Chart rates={rates.clone()} dark_mode={theme_handle.effective_theme == Theme::Dark} />
                    </section>

                    // Carbon tracking
                    {
                        match &*carbon_state {
                            CarbonDataState::Loading => html! {
                                <section class="carbon-section">
                                    <h2>{"Grid Carbon Intensity"}</h2>
                                    <p>{"Loading carbon intensity data..."}</p>
                                </section>
                            },
                            CarbonDataState::Loaded(carbon_data) => html! {
                                <section class="carbon-section">
                                    <h2>{"Grid Carbon Intensity"}</h2>
                                    <CarbonDisplay data={carbon_data.clone()} />
                                </section>
                            },
                            CarbonDataState::Error(err) => html! {
                                <section class="carbon-section">
                                    <h2>{"Grid Carbon Intensity"}</h2>
                                    <p class="error">{format!("Error loading carbon data: {}", err)}</p>
                                </section>
                            },
                        }
                    }
                }
                <section class="status-section">
                    <h2>{"API Status"}</h2>
                    <Status state={(*state).clone()} />
                </section>
            </main>

            <style>
                {include_str!("style.css")}
            </style>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
