use crate::models::load_state::LoadState;
use crate::models::rates::Rates;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusProps {
    pub state: LoadState<Rates>,
}

#[function_component(Status)]
pub fn status(props: &StatusProps) -> Html {
    match &props.state {
        LoadState::Loading => html! {
            <div class="status loading" role="status" aria-live="polite" aria-label="Loading data">
                <div class="spinner" aria-hidden="true"></div>
                <p>{"Loading data..."}</p>
            </div>
        },
        LoadState::Loaded(_) => html! {
            <div class="status success" role="status" aria-live="polite">
                <p>{"✅ Data loaded successfully"}</p>
            </div>
        },
        LoadState::Error(msg) => html! {
            <div class="status error" role="alert" aria-live="assertive">
                <p>{"❌ Error: "}{msg}</p>
            </div>
        },
    }
}
