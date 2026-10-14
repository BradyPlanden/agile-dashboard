use chrono::{DateTime, Utc};
use yew::prelude::*;

use crate::utils::time::london_hhmm;

#[derive(Properties, PartialEq)]
pub struct StatusProps {
    pub updated_at: Option<DateTime<Utc>>,
    pub on_refresh: Callback<()>,
}

/// When prices were last fetched, with a button to fetch everything again now.
#[function_component(Status)]
pub fn status(props: &StatusProps) -> Html {
    html! {
        <p class="status-line">
            <span role="status">
                { props.updated_at.map_or_else(
                    || "Fetching latest prices…".to_owned(),
                    |at| format!("Prices updated {}", london_hhmm(at)),
                ) }
            </span>
            <button type="button" class="link-button" onclick={props.on_refresh.reform(|_| ())}>
                { "Refresh" }
            </button>
        </p>
    }
}
