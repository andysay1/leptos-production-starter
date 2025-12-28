use crate::handlers::{error_response, RequestIdExtractor};
use crate::security;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::models::User;
use domain::ports::UserRepository;
use serde::Deserialize;
use shared::dto::{PaginatedResponse, UserResponse};
use shared::error::AppError;
use shared::types::UserRole;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

#[instrument(skip(state, headers))]
pub async fn list_users(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    headers: HeaderMap,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    let Some(token) = security::bearer_token(&headers) else {
        return error_response(AppError::Unauthorized, &request_id.0).into_response();
    };

    let claims = match security::decode_access_token(&token, &state.config.auth) {
        Ok(c) => c,
        Err(err) => return error_response(err, &request_id.0).into_response(),
    };

    if claims.role != UserRole::Admin {
        return error_response(AppError::Forbidden, &request_id.0).into_response();
    }

    let (users, total) = match state
        .db
        .list_users(pagination.page, pagination.per_page)
        .await
    {
        Ok(data) => data,
        Err(err) => return error_response(err, &request_id.0).into_response(),
    };

    let resp = PaginatedResponse {
        items: users.into_iter().map(to_user_response).collect(),
        total,
        page: pagination.page,
        per_page: pagination.per_page,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

fn to_user_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id,
        email: user.email,
        role: user.role,
        created_at: user.created_at,
    }
}
