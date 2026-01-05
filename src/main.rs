use yew::prelude::*;

mod components;
mod config;
mod hooks;
mod models;
mod services;

use components::chart::Chart;
use components::status::Status;
use components::summary::Summary;
use components::tracker_display::TrackerDisplay;
use components::{TraceBanner, compute_means};
use hooks::use_historical_rates::use_historical_rates;
use hooks::use_rates::use_rates;
use hooks::use_tracker::use_tracker_rates;

#[function_component(App)]
fn app() -> Html {
    let state = use_rates();
    let historical_state = use_historical_rates();
    let tracker_state = use_tracker_rates();

    // Transform historical Agile rates to banner values using memoization
    let banner_values = use_memo(historical_state.clone(), |state| {
        match state.data() {
            Some(rates) => {
                let grouped = rates.grouped_by_half_hour_slot();
                compute_means(&grouped)
            }
            None => vec![], // Empty during Loading/Error
        }
    });

    html! {
        <div class="app-container">
            <header class="app-header">
                <h1>{"Octopus Agile Dashboard"}</h1>
            </header>

            <main class="app-main">
                // Banner section - only show when historical data is loaded and values exist
                if let Some(_rates) = historical_state.data() {
                    if !banner_values.is_empty() {
                        <section class="banner-section">
                            <TraceBanner
                                values={(*banner_values).clone()}
                                height={100}
                                color="#3b82f6"
                                stroke_width={2.0}
                                smooth={true}
                            />
                        </section>
                    }
                }

                if let Some(rates) = state.data() {
                    <section class="data-section">
                        <h2>{"Data Summary"}</h2>
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

                    <section class="chart-section">
                        <h2>{"Energy Price Distribution"}</h2>
                        <Chart rates={rates.clone()} />
                    </section>
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
