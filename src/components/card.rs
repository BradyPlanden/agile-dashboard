use std::rc::Rc;

use yew::prelude::*;

use crate::models::load_state::LoadState;
use crate::utils::time::london_hhmm;

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Html,
}

/// A titled dashboard card.
#[function_component(Card)]
pub fn card(props: &CardProps) -> Html {
    html! {
        <section class={classes!("card", props.class.clone())}>
            <h2>{ &props.title }</h2>
            { props.children.clone() }
        </section>
    }
}

/// `content` for the data, or a placeholder of the same footprint while it loads.
pub fn skeleton_or<T>(data: Option<&Rc<T>>, content: impl FnOnce(&Rc<T>) -> Html) -> Html {
    data.map_or_else(
        || html! { <div class="skeleton" aria-hidden="true" /> },
        content,
    )
}

/// Renders every [`LoadState`]: a skeleton, an error, or `content` with a notice when the
/// data is stale.
pub fn load_view<T>(
    state: &LoadState<T>,
    what: &str,
    content: impl FnOnce(&Rc<T>) -> Html,
) -> Html {
    match state {
        LoadState::Loading | LoadState::Loaded(_) => skeleton_or(state.data(), content),
        LoadState::Failed(error) => html! {
            <p class="notice notice-error" role="alert">
                { format!("Couldn't load {what}: {error}.") }
            </p>
        },
        LoadState::Stale { snapshot, error } => html! {
            <>
                <p class="notice notice-warning" role="status">
                    { format!(
                        "Couldn't refresh {what} ({error}). Showing data from {}.",
                        london_hhmm(snapshot.fetched_at)
                    ) }
                </p>
                { content(&snapshot.data) }
            </>
        },
    }
}
