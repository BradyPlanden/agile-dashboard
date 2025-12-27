use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ApiError(String),
    DataError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ApiError(msg) => write!(f, "API Error: {msg}"),
            AppError::DataError(msg) => write!(f, "Data Error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}
