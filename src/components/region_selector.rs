use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::models::region::Region;

#[derive(Properties, PartialEq)]
pub struct RegionSelectorProps {
    pub region: Region,
    pub on_change: Callback<Region>,
}

/// Drop-down for choosing the electricity distribution region.
#[function_component(RegionSelector)]
pub fn region_selector(props: &RegionSelectorProps) -> Html {
    let onchange = {
        let on_change = props.on_change.clone();
        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            if let Ok(region) = select.value().parse() {
                on_change.emit(region);
            }
        })
    };

    html! {
        <label class="select-wrap">
            <span class="sr-only">{ "Electricity region" }</span>
            <select class="control region-select" {onchange}>
                { for Region::ALL.into_iter().map(|region| html! {
                    <option value={region.code()} selected={region == props.region}>
                        { region.to_string() }
                    </option>
                }) }
            </select>
        </label>
    }
}
