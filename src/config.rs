//! Application-wide tuning constants.

/// How often live rates and carbon intensity are refreshed while the page is visible.
pub const POLL_INTERVAL_MS: u32 = 10 * 60 * 1000;

/// How often the price history is re-checked. Usually a cache hit; the window only moves at
/// London midnight.
pub const HISTORY_POLL_INTERVAL_MS: u32 = 60 * 60 * 1000;

/// Delay before retrying a failed refresh. Doubles per consecutive failure, capped at the
/// polling interval.
pub const ERROR_RETRY_BASE_MS: u32 = 30_000;

/// Maximum time to wait for an HTTP request before aborting it.
pub const REQUEST_TIMEOUT_MS: u32 = 30_000;

/// Maximum attempts for a request that keeps being rate limited (HTTP 429).
pub const MAX_RATE_LIMIT_ATTEMPTS: u32 = 4;

/// Whole days of Agile history shown in the banner.
pub const HISTORY_DAYS: u64 = 31;
