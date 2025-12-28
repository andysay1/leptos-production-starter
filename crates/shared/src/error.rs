use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidationErrors;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found")]
    NotFound,
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "validation_error",
            AppError::Unauthorized => "unauthorized",
            AppError::Forbidden => "forbidden",
            AppError::NotFound => "not_found",
            AppError::Conflict(_) => "conflict",
            AppError::RateLimited => "rate_limited",
            AppError::Config(_) => "config_error",
            AppError::Unavailable(_) => "unavailable",
            AppError::Internal(_) => "internal_error",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            code: self.code().to_string(),
            message: self.to_string(),
            details: None,
            request_id,
        }
    }

    pub fn with_details(self, details: impl Into<String>) -> Self {
        match self {
            AppError::Validation(_) => AppError::Validation(details.into()),
            AppError::Conflict(_) => AppError::Conflict(details.into()),
            AppError::Config(_) => AppError::Config(details.into()),
            AppError::Unavailable(_) => AppError::Unavailable(details.into()),
            AppError::Internal(_) => AppError::Internal(details.into()),
            other => other,
        }
    }

    pub fn config(details: impl Into<String>) -> Self {
        AppError::Config(details.into())
    }

    pub fn internal(details: impl Into<String>) -> Self {
        AppError::Internal(details.into())
    }
}

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        AppError::Validation(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub request_id: Option<String>,
}
