use gloo::events::EventListener;
use gloo_storage::Storage;
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsCast;
use yew::prelude::*;

/// Theme enum representing user's theme preference
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto, // Follow system preference
}

/// Handle returned by use_theme hook
#[derive(Clone, PartialEq)]
pub struct ThemeHandle {
    pub theme: Theme,           // User's preference
    pub effective_theme: Theme, // Resolved theme
    pub toggle: Callback<()>,
    pub set_theme: Callback<Theme>,
}

/// Custom hook for theme management
#[hook]
pub fn use_theme() -> ThemeHandle {
    // Load user preference from localStorage, fallback to Auto
    let theme = use_state(|| load_theme_preference().unwrap_or(Theme::Auto));

    // Detect system preference
    let system_preference = use_state(|| detect_system_preference());

    // Compute effective theme (resolve Auto to Light/Dark)
    let effective_theme = match *theme {
        Theme::Auto => *system_preference,
        other => other,
    };

    // Effect: Apply theme to DOM
    {
        let effective_theme = effective_theme;
        use_effect_with(effective_theme, move |theme| {
            apply_theme_to_dom(*theme);
            || ()
        });
    }

    // Effect: Listen to system preference changes
    {
        let system_preference = system_preference.clone();
        use_effect_with((), move |_| {
            let listener = setup_media_query_listener(system_preference.setter());
            move || drop(listener)
        });
    }

    // Effect: Persist theme to localStorage
    {
        let theme_value = *theme;
        use_effect_with(theme_value, move |theme| {
            save_theme_preference(*theme);
            || ()
        });
    }

    // Toggle callback: switches between Light and Dark
    let toggle = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let new_theme = match *theme {
                Theme::Dark => Theme::Light,
                _ => Theme::Dark,
            };
            theme.set(new_theme);
        })
    };

    // Set theme callback
    let set_theme = {
        let theme = theme.clone();
        Callback::from(move |new_theme| theme.set(new_theme))
    };

    ThemeHandle {
        theme: *theme,
        effective_theme,
        toggle,
        set_theme,
    }
}

/// Detect system's preferred color scheme
fn detect_system_preference() -> Theme {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mq| {
            if mq.matches() {
                Theme::Dark
            } else {
                Theme::Light
            }
        })
        .unwrap_or(Theme::Light)
}

/// Apply theme to DOM by setting data-theme attribute on <html>
fn apply_theme_to_dom(theme: Theme) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(html) = document.document_element() {
            let theme_str = match theme {
                Theme::Dark => "dark",
                Theme::Light => "light",
                Theme::Auto => "light", // Auto should already be resolved
            };
            let _ = html.set_attribute("data-theme", theme_str);
        }
    }
}

/// Load theme preference from localStorage
fn load_theme_preference() -> Option<Theme> {
    match gloo_storage::LocalStorage::get("theme") {
        Ok(theme) => Some(theme),
        Err(_) => {
            web_sys::console::warn_1(
                &"localStorage unavailable or read failed, using default theme".into(),
            );
            None
        }
    }
}

/// Save theme preference to localStorage
fn save_theme_preference(theme: Theme) {
    if let Err(e) = gloo_storage::LocalStorage::set("theme", theme) {
        web_sys::console::warn_1(&format!("Failed to save theme: {:?}", e).into());
    }
}

/// Setup MediaQueryList event listener for system preference changes
fn setup_media_query_listener(setter: UseStateSetter<Theme>) -> Option<EventListener> {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mq| {
            let target = mq.clone().dyn_into::<web_sys::EventTarget>().unwrap();
            EventListener::new(&target, "change", move |_event| {
                setter.set(detect_system_preference());
            })
        })
}
