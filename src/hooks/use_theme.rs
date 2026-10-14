use gloo_events::EventListener;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::utils::log;

const STORAGE_KEY: &str = "theme";
const DARK_QUERY: &str = "(prefers-color-scheme: dark)";

/// The user's theme choice. Stored as `"auto"`, `"light"` or `"dark"`, which the boot
/// script in `index.html` also reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Auto,
    Light,
    Dark,
}

/// The colour scheme actually applied.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl From<ColorScheme> for ThemePreference {
    fn from(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self::Light,
            ColorScheme::Dark => Self::Dark,
        }
    }
}

impl ThemePreference {
    /// The scheme this preference shows, given the system's.
    pub const fn resolve(self, system: ColorScheme) -> ColorScheme {
        match self {
            Self::Auto => system,
            Self::Light => ColorScheme::Light,
            Self::Dark => ColorScheme::Dark,
        }
    }

    /// The next preference in the toggle cycle. The first click always changes what's shown,
    /// and the cycle passes through both explicit schemes before returning to `Auto`.
    pub const fn next(self, system: ColorScheme) -> Self {
        match self {
            Self::Auto => match system.opposite() {
                ColorScheme::Light => Self::Light,
                ColorScheme::Dark => Self::Dark,
            },
            Self::Light if matches!(system, ColorScheme::Dark) => Self::Dark,
            Self::Dark if matches!(system, ColorScheme::Light) => Self::Light,
            Self::Light | Self::Dark => Self::Auto,
        }
    }
}

/// Theme state returned by [`use_theme`].
#[derive(Clone, PartialEq)]
pub struct ThemeHandle {
    pub preference: ThemePreference,
    pub scheme: ColorScheme,
    pub cycle: Callback<()>,
}

/// Persists the theme preference, follows system changes in `Auto`, and applies the
/// resolved scheme to `<html data-theme>`.
#[hook]
pub fn use_theme() -> ThemeHandle {
    let preference = use_state(load_preference);
    let system = use_state(system_scheme);
    let scheme = preference.resolve(*system);

    use_effect_with(scheme, |scheme| {
        apply_to_document(*scheme);
    });

    use_effect_with(*preference, |preference| {
        if let Err(error) = LocalStorage::set(STORAGE_KEY, preference) {
            log::warn(&format!("Failed to save theme: {error}"));
        }
    });

    {
        let setter = system.setter();
        use_effect_with((), move |()| {
            let listener = dark_query().map(|query| {
                EventListener::new(&query, "change", move |_| setter.set(system_scheme()))
            });
            move || drop(listener)
        });
    }

    let cycle = {
        let preference = preference.clone();
        let system = *system;
        Callback::from(move |()| preference.set(preference.next(system)))
    };

    ThemeHandle {
        preference: *preference,
        scheme,
        cycle,
    }
}

fn load_preference() -> ThemePreference {
    LocalStorage::get(STORAGE_KEY).unwrap_or_default()
}

fn dark_query() -> Option<web_sys::MediaQueryList> {
    web_sys::window()?.match_media(DARK_QUERY).ok().flatten()
}

fn system_scheme() -> ColorScheme {
    if dark_query().is_some_and(|query| query.matches()) {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    }
}

fn apply_to_document(scheme: ColorScheme) {
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        let _ = root.set_attribute("data-theme", scheme.as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_toggle_from_auto_flips_the_visible_scheme() {
        assert_eq!(
            ThemePreference::Auto.next(ColorScheme::Light),
            ThemePreference::Dark
        );
        assert_eq!(
            ThemePreference::Auto.next(ColorScheme::Dark),
            ThemePreference::Light
        );
    }

    #[test]
    fn cycle_visits_both_schemes_then_returns_to_auto() {
        for system in [ColorScheme::Light, ColorScheme::Dark] {
            let first = ThemePreference::Auto.next(system);
            let second = first.next(system);
            let third = second.next(system);

            assert_ne!(first.resolve(system), second.resolve(system));
            assert_eq!(third, ThemePreference::Auto);
        }
    }

    #[test]
    fn stored_values_match_the_boot_script() {
        assert_eq!(
            serde_json::to_string(&ThemePreference::Dark).unwrap(),
            r#""dark""#
        );
        assert_eq!(
            serde_json::from_str::<ThemePreference>(r#""auto""#).unwrap(),
            ThemePreference::Auto
        );
    }
}
