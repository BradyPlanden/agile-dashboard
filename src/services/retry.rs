use crate::models::error::AppError;
use gloo_timers::future::TimeoutFuture;
use std::future::Future;

/// Retries an async operation with exponential backoff for rate-limited requests.
///
/// # Arguments
///
/// * `operation` - A closure that returns a Future resolving to `Result<T, AppError>`
/// * `max_attempts` - Maximum number of retry attempts
///
/// # Returns
///
/// The successful result, or the last error encountered
///
/// # Behavior
///
/// - Initial delay: 100ms
/// - Backoff multiplier: 5x (100ms → 500ms → 2500ms → ...)
/// - Only retries on `AppError::RateLimited`
/// - All other errors immediately propagate
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_attempts: u32,
) -> Result<T, AppError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let mut delay_ms = 100;

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(AppError::RateLimited) if attempt < max_attempts => {
                gloo::console::warn!(&format!(
                    "Rate limited, retrying in {}ms (attempt {}/{})",
                    delay_ms, attempt, max_attempts
                ));
                TimeoutFuture::new(delay_ms).await;
                delay_ms *= 5; // Exponential backoff: 100ms, 500ms, 2500ms, ...
            }
            Err(e) => return Err(e),
        }
    }

    Err(AppError::RateLimited)
}
