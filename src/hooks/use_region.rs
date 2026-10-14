use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;

use crate::models::region::Region;
use crate::utils::log;

const STORAGE_KEY: &str = "region";

/// Region state returned by [`use_region`].
#[derive(Clone, PartialEq)]
pub struct RegionHandle {
    pub region: Region,
    pub set_region: Callback<Region>,
}

/// The selected region, persisted to local storage.
#[hook]
pub fn use_region() -> RegionHandle {
    let region = use_state(|| load_region().unwrap_or_default());

    use_effect_with(*region, |region| {
        if let Err(error) = LocalStorage::set(STORAGE_KEY, region.code()) {
            log::warn(&format!("Failed to save region: {error}"));
        }
    });

    RegionHandle {
        region: *region,
        set_region: Callback::from(move |new_region| region.set(new_region)),
    }
}

/// Parses rather than deserializes, so arbitrary stored strings are rejected.
fn load_region() -> Option<Region> {
    LocalStorage::get::<String>(STORAGE_KEY).ok()?.parse().ok()
}
