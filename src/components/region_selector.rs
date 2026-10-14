use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::models::region::Region;

#[derive(Properties, PartialEq)]
pub struct RegionSelectorProps {
    pub region: Region,
    pub on_change: Callback<Region>,
}

/// Region selector dropdown component
#[function_component(RegionSelector)]
pub fn region_selector(props: &RegionSelectorProps) -> Html {
    let on_change = {
        let callback = props.on_change.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let value = target.value();
            if let Ok(region) = value.parse::<Region>() {
                callback.emit(region);
            }
        })
    };

    html! {
        <select
            class="region-selector"
            onchange={on_change}
            aria-label="Select electricity region"
            title="Select electricity region"
        >
            {
                Region::ALL.iter().map(|r| {
                    let code = r.code();
                    let label = r.to_string();
                    let selected = *r == props.region;
                    html! {
                        <option value={code} {selected}>{label}</option>
                    }
                }).collect::<Html>()
            }
        </select>
    }
}
