pub mod auth;
pub mod health;
pub mod users;

use axum::{http::StatusCode, Json};
use shared::error::{AppError, ErrorResponse};
use shared::types::RequestId;

pub fn error_response(err: AppError, request_id: &RequestId) -> (StatusCode, Json<ErrorResponse>) {
    let response = err.to_response(Some(request_id.clone()));
    (err.status(), Json(response))
}

pub use health::RequestIdExtractor;
