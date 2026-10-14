use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use serde::de::DeserializeOwned;
use web_sys::AbortSignal;

use crate::config::{MAX_RATE_LIMIT_ATTEMPTS, REQUEST_TIMEOUT_MS};
use crate::models::error::AppError;
use crate::utils::log;

/// GETs `url` and decodes its JSON body, backing off and retrying while rate limited.
pub async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, AppError> {
    let mut delay_ms = 100;
    let mut attempt = 1;
    loop {
        match get_json_once(url).await {
            Err(AppError::RateLimited) if attempt < MAX_RATE_LIMIT_ATTEMPTS => {
                log::warn(&format!(
                    "Rate limited, retrying in {delay_ms}ms (attempt {attempt}/{MAX_RATE_LIMIT_ATTEMPTS})"
                ));
                TimeoutFuture::new(delay_ms).await;
                delay_ms *= 5;
                attempt += 1;
            }
            result => return result,
        }
    }
}

async fn get_json_once<T: DeserializeOwned>(url: &str) -> Result<T, AppError> {
    // Unlike racing a timer, the signal also cancels the underlying fetch.
    let signal = AbortSignal::timeout_with_u32(REQUEST_TIMEOUT_MS);
    let response = Request::get(url)
        .abort_signal(Some(&signal))
        .send()
        .await
        .map_err(classify)?;

    if !response.ok() {
        return Err(error_for_status(response.status()));
    }
    response.json().await.map_err(classify)
}

const fn error_for_status(status: u16) -> AppError {
    match status {
        429 => AppError::RateLimited,
        status => AppError::Http(status),
    }
}

fn classify(error: gloo_net::Error) -> AppError {
    match error {
        gloo_net::Error::JsError(error) if error.name == "TimeoutError" => AppError::Timeout,
        gloo_net::Error::JsError(error) => AppError::Network(error.message),
        gloo_net::Error::SerdeError(error) => AppError::Parse(error.to_string()),
        gloo_net::Error::GlooError(message) => AppError::Network(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_decode_and_gloo_errors() {
        let decode_error = serde_json::from_str::<u32>("not json").unwrap_err();

        assert!(matches!(
            classify(gloo_net::Error::SerdeError(decode_error)),
            AppError::Parse(_)
        ));
        assert_eq!(
            classify(gloo_net::Error::GlooError("offline".to_owned())),
            AppError::Network("offline".to_owned())
        );
    }

    #[test]
    fn maps_too_many_requests_to_rate_limited() {
        assert_eq!(error_for_status(429), AppError::RateLimited);
        assert_eq!(error_for_status(503), AppError::Http(503));
    }
}
