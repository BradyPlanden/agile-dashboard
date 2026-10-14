use yew::prelude::*;

use crate::hooks::use_theme::ThemePreference;

#[derive(Properties, PartialEq)]
pub struct ThemeToggleProps {
    pub preference: ThemePreference,
    pub on_cycle: Callback<()>,
}

/// Cycles the theme preference between automatic, light and dark.
#[function_component(ThemeToggle)]
pub fn theme_toggle(props: &ThemeToggleProps) -> Html {
    let (icon, label) = match props.preference {
        ThemePreference::Auto => ("🌓", "Theme: automatic"),
        ThemePreference::Light => ("☀️", "Theme: light"),
        ThemePreference::Dark => ("🌙", "Theme: dark"),
    };

    html! {
        <button
            type="button"
            class="control icon-button"
            onclick={props.on_cycle.reform(|_| ())}
            title={label}
        >
            <span aria-hidden="true">{ icon }</span>
            <span class="sr-only">{ format!("{label}. Change theme") }</span>
        </button>
    }
}
