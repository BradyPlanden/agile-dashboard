/// Configuration constants for the application
pub struct Config;

impl Config {
    /// Enable automatic data refresh polling
    pub const ENABLE_AUTO_REFRESH: bool = true;

    /// Polling interval in milliseconds (10 minutes = 600,000ms)
    pub const POLLING_INTERVAL_MS: u32 = 600_000;
}
