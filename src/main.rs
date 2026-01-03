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
use hooks::use_rates::use_rates;
use hooks::use_tracker::use_tracker_rates;

#[function_component(App)]
fn app() -> Html {
    let state = use_rates();
    let tracker_state = use_tracker_rates();

    html! {
        <div class="app-container">
            <header class="app-header">
                <h1>{"Octopus Agile Dashboard"}</h1>
            </header>

            <main class="app-main">
                <section class="status-section">
                    <h2>{"API Status"}</h2>
                    <Status state={(*state).clone()} />
                </section>

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
