use yew::prelude::*;

use crate::components::{
    CarbonDisplay, Card, CheapestPeriod, HistoryBanner, NowPanel, PriceChart, RegionSelector,
    Status, Summary, ThemeToggle, TrackerDisplay, load_view, skeleton_or,
};
use crate::hooks::use_carbon::use_carbon_intensity;
use crate::hooks::use_historical_rates::use_historical_rates;
use crate::hooks::use_now::use_now;
use crate::hooks::use_rates::use_rates;
use crate::hooks::use_region::{RegionHandle, use_region};
use crate::hooks::use_theme::use_theme;
use crate::hooks::use_tracker::use_tracker_rates;
use crate::models::load_state::LoadState;

#[function_component(App)]
pub fn app() -> Html {
    let RegionHandle { region, set_region } = use_region();
    let theme = use_theme();
    let now = use_now();
    let agile = use_rates(region);
    let tracker = use_tracker_rates(region);
    let carbon = use_carbon_intensity();
    let history = use_historical_rates(region);

    let refresh_all = {
        let refreshes = [
            agile.refresh.clone(),
            tracker.refresh.clone(),
            carbon.refresh.clone(),
            history.refresh.clone(),
        ];
        Callback::from(move |()| refreshes.iter().for_each(|refresh| refresh.emit(())))
    };

    // The history banner is decorative, so it disappears rather than showing an error.
    let history_card = match &history.state {
        LoadState::Failed(_) => html! {},
        state => html! {
            <section class="card history-card" aria-label="Agile price history">
                { skeleton_or(state.data(), |rates| html! {
                    <HistoryBanner rates={rates.clone()} {region} />
                }) }
            </section>
        },
    };
    let agile_rates = agile.state.data();

    html! {
        <div class="app">
            <header class="app-header">
                <h1>{ "Octopus Agile Dashboard" }</h1>
                <div class="header-controls">
                    if let Some(rates) = agile_rates {
                        <CheapestPeriod rates={rates.clone()} {now} />
                    }
                    <RegionSelector {region} on_change={set_region} />
                    <ThemeToggle preference={theme.preference} on_cycle={theme.cycle} />
                </div>
            </header>

            <main class="app-main">
                { history_card }

                <section class="card now-card" aria-label="Current Agile price">
                    { load_view(&agile.state, "Agile prices", |rates| html! {
                        <NowPanel rates={rates.clone()} {now} />
                    }) }
                </section>

                // An Agile failure is reported once, above, rather than in every Agile card.
                if !matches!(agile.state, LoadState::Failed(_)) {
                    <Card title="Energy Price Distribution" class="chart-card">
                        { skeleton_or(agile_rates, |rates| html! {
                            <PriceChart rates={rates.clone()} {now} />
                        }) }
                    </Card>
                    <Card title="Agile prices" class="agile-card">
                        { skeleton_or(agile_rates, |rates| html! {
                            <Summary rates={rates.clone()} {now} />
                        }) }
                    </Card>
                }

                <Card title="Tracker" class="tracker-card">
                    { load_view(&tracker.state, "Tracker prices", |rates| html! {
                        <TrackerDisplay rates={rates.clone()} {now} />
                    }) }
                </Card>

                <Card title="Grid carbon intensity" class="carbon-card">
                    { load_view(&carbon.state, "carbon intensity", |data| html! {
                        <CarbonDisplay data={data.clone()} />
                    }) }
                </Card>
            </main>

            <footer class="app-footer">
                <Status
                    updated_at={agile.state.snapshot().map(|snapshot| snapshot.fetched_at)}
                    on_refresh={refresh_all}
                />
                <p class="attribution">
                    { "Prices from the " }
                    <a href="https://developer.octopus.energy/">{ "Octopus Energy API" }</a>
                    { ". Carbon intensity from " }
                    <a href="https://carbonintensity.org.uk/">{ "carbonintensity.org.uk" }</a>
                    { "." }
                </p>
            </footer>
        </div>
    }
}
