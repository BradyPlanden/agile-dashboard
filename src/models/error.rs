#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Data error: {0}")]
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
