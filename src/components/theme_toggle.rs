use yew::prelude::*;

use crate::hooks::use_theme::Theme;

#[derive(Properties, PartialEq)]
pub struct ThemeToggleProps {
    pub effective_theme: Theme,
    pub on_toggle: Callback<()>,
}

/// Theme toggle button component
#[function_component(ThemeToggle)]
pub fn theme_toggle(props: &ThemeToggleProps) -> Html {
    // Determine icon and label based on effective theme
    let (icon, label) = match props.effective_theme {
        Theme::Dark => ("☀️", "Switch to light mode"),
        Theme::Light => ("🌙", "Switch to dark mode"),
        Theme::Auto => ("🌓", "Auto theme"),
    };

    let onclick = {
        let toggle = props.on_toggle.clone();
        Callback::from(move |_| toggle.emit(()))
    };

    html! {
        <button
            class="theme-toggle"
            {onclick}
            aria-label={label}
            aria-pressed={if props.effective_theme == Theme::Dark { "true" } else { "false" }}
            title={label}
        >
            <span aria-hidden="true">{icon}</span>
            <span class="sr-only">{label}</span>
        </button>
    }
}
