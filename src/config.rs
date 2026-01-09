/// Configuration constants for the application
pub struct Config;

impl Config {
    /// Enable automatic data refresh polling
    pub const ENABLE_AUTO_REFRESH: bool = true;

    /// Polling interval in milliseconds (10 minutes = 600,000ms)
    pub const POLLING_INTERVAL_MS: u32 = 600_000;

    /// Delay between pagination requests (ms) to avoid rate limiting
    pub const PAGINATION_DELAY_MS: u32 = 5;

    /// Maximum retry attempts for rate-limited requests
    pub const MAX_RETRY_ATTEMPTS: u32 = 10;
}
