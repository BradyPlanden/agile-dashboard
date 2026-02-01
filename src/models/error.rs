#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    #[error("API Error: {0}")]
    ApiError(String),

    #[error("Data Error: {0}")]
    DataError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
