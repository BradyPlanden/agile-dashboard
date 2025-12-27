use yew::prelude::*;

mod components;
mod hooks;
mod models;
mod services;

use components::chart::Chart;
use components::status::Status;
use components::summary::Summary;
use hooks::use_rates::use_rates;

#[function_component(App)]
fn app() -> Html {
    let state = use_rates();

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
