use yew::prelude::*;

use crate::hooks::use_theme::{Theme, use_theme};

/// Theme toggle button component
#[function_component(ThemeToggle)]
pub fn theme_toggle() -> Html {
    let theme_handle = use_theme();

    // Determine icon and label based on effective theme
    let (icon, label) = match theme_handle.effective_theme {
        Theme::Dark => ("☀️", "Switch to light mode"),
        Theme::Light => ("🌙", "Switch to dark mode"),
        Theme::Auto => ("🌓", "Auto theme"), // Exhaustive match: shouldn't happen since effective_theme is resolved
    };

    let onclick = {
        let toggle = theme_handle.toggle;
        Callback::from(move |_| toggle.emit(()))
    };

    html! {
        <button
            class="theme-toggle"
            {onclick}
            aria-label={label}
            title={label}
        >
            {icon}
        </button>
    }
}
