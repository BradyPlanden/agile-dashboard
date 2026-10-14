use gloo_events::EventListener;
use yew::prelude::*;

/// Whether the page is currently visible, updating on `visibilitychange`.
#[hook]
pub fn use_page_visible() -> bool {
    let visible = use_state(is_page_visible);
    {
        let setter = visible.setter();
        use_effect_with((), move |()| {
            let listener = web_sys::window()
                .and_then(|window| window.document())
                .map(|document| {
                    EventListener::new(&document, "visibilitychange", move |_| {
                        setter.set(is_page_visible());
                    })
                });
            move || drop(listener)
        });
    }
    *visible
}

pub fn is_page_visible() -> bool {
    web_sys::window()
        .and_then(|window| window.document())
        .is_none_or(|document| !document.hidden())
}
