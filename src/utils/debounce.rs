use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{HtmlElement, ResizeObserver};

pub struct DebouncedResizeObserver {
    observer: ResizeObserver,
    _callback: Closure<dyn FnMut()>,
}

impl Drop for DebouncedResizeObserver {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

pub fn create_debounced_resize_observer<F>(
    element: &HtmlElement,
    callback: F,
    delay_ms: u32,
) -> Result<DebouncedResizeObserver, wasm_bindgen::JsValue>
where
    F: Fn() + 'static,
{
    let timeout_handle: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));
    let callback = Rc::new(callback);

    let observer_callback = {
        let timeout_handle = timeout_handle.clone();
        let callback = callback.clone();

        Closure::wrap(Box::new(move || {
            if let Some(handle) = timeout_handle.borrow_mut().take() {
                drop(handle);
            }

            let cb = callback.clone();
            let handle = Timeout::new(delay_ms, move || cb());
            *timeout_handle.borrow_mut() = Some(handle);
        }) as Box<dyn FnMut()>)
    };

    let observer = ResizeObserver::new(observer_callback.as_ref().unchecked_ref())?;
    observer.observe(element);

    Ok(DebouncedResizeObserver {
        observer,
        _callback: observer_callback,
    })
}
