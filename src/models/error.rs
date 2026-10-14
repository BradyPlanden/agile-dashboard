/// Errors from fetching or interpreting remote data, shown to the user.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    #[error("network error: {0}")]
    Network(String),

    #[error("request timed out")]
    Timeout,

    #[error("rate limited by the API")]
    RateLimited,

    #[error("API returned HTTP {0}")]
    Http(u16),

    #[error("unexpected response: {0}")]
    Parse(String),

    #[error("{0}")]
    MissingData(&'static str),
}
