//! Application-wide tuning constants.

/// How often live rates and carbon intensity are refreshed while the page is visible.
pub const POLL_INTERVAL_MS: u32 = 10 * 60 * 1000;

/// How often the price history is re-checked, which also caps its retry backoff.
///
/// Usually a cache hit: the history only changes when its window moves at London midnight,
/// and that starts a fetch of its own.
pub const HISTORY_POLL_INTERVAL_MS: u32 = 60 * 60 * 1000;

/// Delay before retrying a failed refresh. Doubles per consecutive failure, capped at the
/// polling interval.
pub const ERROR_RETRY_BASE_MS: u32 = 30_000;

/// Maximum time to wait for an HTTP request before aborting it.
pub const REQUEST_TIMEOUT_MS: u32 = 30_000;

/// Maximum attempts for a request that keeps being rate limited (HTTP 429).
pub const MAX_RATE_LIMIT_ATTEMPTS: u32 = 4;

/// How far ahead the header looks for the cheapest half hour.
pub const CHEAPEST_SLOT_LOOKAHEAD_HOURS: i64 = 3;

/// Default and maximum length, in hours, of the cheapest upcoming block beside the current
/// price. The user can change it with a stepper.
pub const CHEAPEST_WINDOW_DEFAULT_HOURS: u8 = 4;
pub const CHEAPEST_WINDOW_MAX_HOURS: u8 = 12;

/// Whole days of Agile history shown in the banner.
pub const HISTORY_DAYS: u64 = 31;
