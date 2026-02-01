use gloo_storage::Storage;
use yew::prelude::*;

use crate::services::api::Region;

/// Handle returned by `use_region` hook
#[derive(Clone, PartialEq)]
pub struct RegionHandle {
    pub region: Region,
    pub set_region: Callback<Region>,
}

/// Custom hook for region management with localStorage persistence
#[hook]
pub fn use_region() -> RegionHandle {
    // Load region from localStorage, fallback to default (Region::C / London)
    let region = use_state(|| load_region_preference().unwrap_or_default());

    // Effect: Persist region to localStorage on change
    {
        let region_value = *region;
        use_effect_with(region_value, move |region| {
            save_region_preference(*region);
            || ()
        });
    }

    // Set region callback
    let set_region = {
        let region = region.clone();
        Callback::from(move |new_region| region.set(new_region))
    };

    RegionHandle {
        region: *region,
        set_region,
    }
}

/// Load region preference from localStorage
fn load_region_preference() -> Option<Region> {
    gloo_storage::LocalStorage::get("region").ok()
}

/// Save region preference to localStorage
fn save_region_preference(region: Region) {
    if let Err(e) = gloo_storage::LocalStorage::set("region", region) {
        web_sys::console::warn_1(&format!("Failed to save region: {e:?}").into());
    }
}
