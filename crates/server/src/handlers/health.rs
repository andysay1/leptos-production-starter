use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use shared::dto::HealthStatus;
use shared::types::RequestId;
use uuid::Uuid;

pub async fn health(
    State(state): State<AppState>,
    _request_id: RequestIdExtractor,
) -> impl axum::response::IntoResponse {
    let body = HealthStatus {
        status: "ok".into(),
        db: false,
        redis: state.redis.is_some(),
        version: env!("CARGO_PKG_VERSION").into(),
    };
    (StatusCode::OK, Json(body))
}

pub async fn ready(
    State(state): State<AppState>,
    _request_id: RequestIdExtractor,
) -> impl axum::response::IntoResponse {
    let db_ok = state.db.ping().await.is_ok();
    let redis_ok = if let Some(client) = &state.redis {
        let mut conn = client.clone();
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .is_ok()
    } else {
        false
    };

    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = HealthStatus {
        status: if db_ok {
            "ok".into()
        } else {
            "degraded".into()
        },
        db: db_ok,
        redis: redis_ok,
        version: env!("CARGO_PKG_VERSION").into(),
    };

    (status, Json(body))
}

#[derive(Clone, Debug)]
pub struct RequestIdExtractor(pub RequestId);

impl<S> axum::extract::FromRequestParts<S> for RequestIdExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send
    {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        async move { Ok(RequestIdExtractor(request_id)) }
    }
}
