use gloo::events::EventListener;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::window;

/// Creates a debounced resize listener that delays execution until after a period of inactivity.
///
/// This prevents performance issues caused by rapidly firing resize events (60+ times/second
/// during window dragging). The callback will only execute after the specified delay has passed
/// with no new resize events.
///
/// # Arguments
///
/// * `callback` - The function to execute after the debounce delay
/// * `delay_ms` - Milliseconds to wait before executing (recommended: 150ms)
///
/// # Returns
///
/// An `EventListener` that must be kept alive for the duration of the component lifecycle.
/// When dropped, the listener is automatically cleaned up.
///
/// # Example
///
/// ```rust,ignore
/// use_effect_with(container_ref.clone(), move |container_ref| {
///     let listener = create_debounced_resize_listener(move || {
///         // This will only run 150ms after the user stops resizing
///         update_dimensions();
///     }, 150);
///
///     move || drop(listener) // Cleanup
/// });
/// ```
pub fn create_debounced_resize_listener<F>(callback: F, delay_ms: u32) -> EventListener
where
    F: Fn() + 'static,
{
    let timeout_handle: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));
    let callback = Rc::new(callback);

    EventListener::new(&window().unwrap(), "resize", move |_| {
        // Cancel pending timeout
        if let Some(handle) = timeout_handle.borrow_mut().take() {
            drop(handle);
        }

        // Schedule new timeout
        let cb = callback.clone();
        let handle = Timeout::new(delay_ms, move || cb());
        *timeout_handle.borrow_mut() = Some(handle);
    })
}
