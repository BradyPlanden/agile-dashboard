use crate::models::error::AppError;
use futures::future::{Either, select};
use futures::pin_mut;
use gloo_timers::future::TimeoutFuture;

/// Classify a reqwest error into an appropriate `AppError`.
pub fn classify_reqwest_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::ApiError("Request timed out".to_string())
    } else if error.is_request() {
        AppError::ApiError("Request could not be sent".to_string())
    } else {
        AppError::ApiError("Network request failed".to_string())
    }
}

/// Map an HTTP status code and response body into an `AppError`.
///
/// Sanitises response bodies so internal API details are not exposed to users.
pub fn error_for_status(status: reqwest::StatusCode, _body: &str) -> AppError {
    match status.as_u16() {
        429 => AppError::RateLimited,
        401 | 403 => AppError::AuthError(format!("Authentication failed: {status}")),
        404 => AppError::NotFound("Resource not found".to_string()),
        400..=499 => AppError::ApiError(format!("Client error {status}")),
        500..=599 => AppError::ApiError(format!("Server error {status}")),
        _ => AppError::ApiError(format!("Unexpected status {status}")),
    }
}

/// Builds the shared `reqwest::Client` used by API services.
pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .build()
        .expect("Failed to create shared HTTP client")
}

/// Sends a request with an application-level timeout that works on wasm.
pub async fn send_with_timeout(
    request: reqwest::RequestBuilder,
) -> Result<reqwest::Response, AppError> {
    timeout(async move { request.send().await.map_err(classify_reqwest_error) }).await
}

async fn timeout<T, Fut>(future: Fut) -> Result<T, AppError>
where
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let timeout = TimeoutFuture::new(crate::config::Config::REQUEST_TIMEOUT_MS);
    pin_mut!(future);
    pin_mut!(timeout);

    match select(future, timeout).await {
        Either::Left((result, _)) => result,
        Either::Right(((), _)) => Err(AppError::ApiError("Request timed out".to_string())),
    }
}
