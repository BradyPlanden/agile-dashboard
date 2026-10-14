//! Console logging that also works in native unit tests.

pub fn warn(message: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::warn_1(&message.into());

    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("warning: {message}");
}
