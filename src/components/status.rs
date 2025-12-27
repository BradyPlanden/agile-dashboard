use crate::hooks::use_rates::DataState;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusProps {
    pub state: DataState,
}

#[function_component(Status)]
pub fn status(props: &StatusProps) -> Html {
    match &props.state {
        DataState::Loading => html! {
            <div class="status loading">
                <div class="spinner"></div>
                <p>{"Loading data..."}</p>
            </div>
        },
        DataState::Loaded(_) => html! {
            <div class="status success">
                <p>{"✅ Data loaded successfully"}</p>
            </div>
        },
        DataState::Error(msg) => html! {
            <div class="status error">
                <p>{"❌ Error: "}{msg}</p>
            </div>
        },
    }
}
