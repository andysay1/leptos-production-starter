use crate::handlers::{error_response, RequestIdExtractor};
use crate::security;
use crate::state::AppState;
use axum::{
    extract::Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use axum_extra::extract::CookieJar;
use domain::models::User;
use domain::ports::UserRepository;
use shared::dto::{LoginRequest, RegisterRequest, TokenResponse, UserResponse};
use shared::error::AppError;
use time::Duration as TimeDuration;
use tracing::instrument;

#[instrument(skip(state, jar, payload))]
pub async fn register(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match state.auth.register(payload, None).await {
        Ok(user) => match issue_session(&state, jar, user).await {
            Ok((jar, tokens)) => (jar, (StatusCode::CREATED, Json(tokens))).into_response(),
            Err(err) => error_response(err, &request_id.0).into_response(),
        },
        Err(err) => error_response(err, &request_id.0).into_response(),
    }
}

#[instrument(skip(state, jar, payload))]
pub async fn login(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.auth.login(payload).await {
        Ok(user) => match issue_session(&state, jar, user).await {
            Ok((jar, tokens)) => (jar, (StatusCode::OK, Json(tokens))).into_response(),
            Err(err) => error_response(err, &request_id.0).into_response(),
        },
        Err(err) => error_response(err, &request_id.0).into_response(),
    }
}

#[instrument(skip(state, jar, payload))]
pub async fn login_form(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    jar: CookieJar,
    Form(payload): Form<LoginRequest>,
) -> impl IntoResponse {
    match state.auth.login(payload).await {
        Ok(user) => match issue_session(&state, jar, user).await {
            Ok((jar, _tokens)) => (jar, Redirect::to("/app")).into_response(),
            Err(err) => error_response(err, &request_id.0).into_response(),
        },
        Err(err) => error_response(err, &request_id.0).into_response(),
    }
}

#[instrument(skip(state, jar, payload))]
pub async fn register_form(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    jar: CookieJar,
    Form(payload): Form<RegisterRequest>,
) -> impl IntoResponse {
    match state.auth.register(payload, None).await {
        Ok(user) => match issue_session(&state, jar, user).await {
            Ok((jar, _tokens)) => (jar, Redirect::to("/app")).into_response(),
            Err(err) => error_response(err, &request_id.0).into_response(),
        },
        Err(err) => error_response(err, &request_id.0).into_response(),
    }
}

#[instrument(skip(state, jar, headers))]
pub async fn refresh(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    jar: CookieJar,
    headers: HeaderMap,
) -> impl IntoResponse {
    let refresh_cookie = match jar.get(&state.config.auth.refresh_cookie_name) {
        Some(c) => c.value().to_string(),
        None => return error_response(AppError::Unauthorized, &request_id.0).into_response(),
    };

    let csrf_cookie = jar
        .get(&state.config.auth.csrf_cookie_name)
        .map(|c| c.value().to_string());
    let csrf_header = headers
        .get("x-csrf-token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let Err(err) = security::verify_csrf(csrf_header.as_deref(), csrf_cookie.as_deref()) {
        return error_response(err, &request_id.0).into_response();
    }

    let token = match state.auth.validate_refresh_token(&refresh_cookie).await {
        Ok(token) => token,
        Err(err) => return error_response(err, &request_id.0).into_response(),
    };

    // rotate token
    let _ = state.auth.logout(&refresh_cookie).await;

    let user = match state.db.find_by_id(token.user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return error_response(AppError::Unauthorized, &request_id.0).into_response(),
        Err(err) => return error_response(err, &request_id.0).into_response(),
    };

    match issue_session(&state, jar, user).await {
        Ok((jar, tokens)) => (jar, (StatusCode::OK, Json(tokens))).into_response(),
        Err(err) => error_response(err, &request_id.0).into_response(),
    }
}

#[instrument(skip(state, jar, headers))]
pub async fn logout(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    jar: CookieJar,
    headers: HeaderMap,
) -> impl IntoResponse {
    let refresh_cookie = match jar.get(&state.config.auth.refresh_cookie_name) {
        Some(c) => c.value().to_string(),
        None => return (jar, StatusCode::NO_CONTENT).into_response(),
    };

    let csrf_cookie = jar
        .get(&state.config.auth.csrf_cookie_name)
        .map(|c| c.value().to_string());
    let csrf_header = headers
        .get("x-csrf-token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let Err(err) = security::verify_csrf(csrf_header.as_deref(), csrf_cookie.as_deref()) {
        return error_response(err, &request_id.0).into_response();
    }

    let _ = state.auth.logout(&refresh_cookie).await;
    let cleared = clear_session(jar, &state.config);
    (cleared, StatusCode::NO_CONTENT).into_response()
}

pub async fn logout_get(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let cleared = clear_session(jar, &state.config);
    (cleared, Redirect::to("/")).into_response()
}

#[instrument(skip(state))]
pub async fn me(
    State(state): State<AppState>,
    request_id: RequestIdExtractor,
    headers: HeaderMap,
    jar: CookieJar,
) -> impl IntoResponse {
    let token = security::bearer_token(&headers).or_else(|| {
        jar.get(&state.config.auth.access_cookie_name)
            .map(|c| c.value().to_string())
    });
    let Some(token) = token else {
        return error_response(AppError::Unauthorized, &request_id.0).into_response();
    };

    let claims = match security::decode_access_token(&token, &state.config.auth) {
        Ok(c) => c,
        Err(err) => return error_response(err, &request_id.0).into_response(),
    };

    let user = match state.db.find_by_id(claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => return error_response(AppError::Unauthorized, &request_id.0).into_response(),
        Err(err) => return error_response(err, &request_id.0).into_response(),
    };

    let body = to_user_response(&user);
    (StatusCode::OK, Json(body)).into_response()
}

async fn issue_session(
    state: &AppState,
    jar: CookieJar,
    user: User,
) -> Result<(CookieJar, TokenResponse), AppError> {
    let access_token = security::sign_access_token(user.id, user.role, &state.config.auth)?;
    let refresh_raw = domain::auth::generate_refresh_token();
    let _stored = state
        .auth
        .store_refresh_token(
            user.id,
            &refresh_raw,
            state.config.auth.refresh_token_ttl_days as i64,
        )
        .await?;

    let csrf_token = security::generate_csrf_token();
    let jar =
        attach_session_cookies(jar, &state.config, &access_token, &refresh_raw, &csrf_token);
    let tokens = TokenResponse {
        access_token,
        user: to_user_response(&user),
        csrf_token,
    };

    Ok((jar, tokens))
}

fn attach_session_cookies(
    jar: CookieJar,
    config: &shared::config::AppConfig,
    access: &str,
    refresh: &str,
    csrf: &str,
) -> CookieJar {
    let access_cookie = security::build_access_cookie(access, config);
    let refresh_cookie = security::build_refresh_cookie(refresh, config);
    let csrf_cookie = security::build_csrf_cookie(csrf, config);
    jar.add(access_cookie).add(refresh_cookie).add(csrf_cookie)
}

fn clear_session(jar: CookieJar, config: &shared::config::AppConfig) -> CookieJar {
    let refresh =
        axum_extra::extract::cookie::Cookie::build((config.auth.refresh_cookie_name.clone(), ""))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .secure(config.auth.cookie_secure)
            .domain(config.server.cookie_domain.clone())
            .path("/")
            .max_age(TimeDuration::seconds(0))
            .build();
    let access =
        axum_extra::extract::cookie::Cookie::build((config.auth.access_cookie_name.clone(), ""))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .secure(config.auth.cookie_secure)
            .domain(config.server.cookie_domain.clone())
            .path("/")
            .max_age(TimeDuration::seconds(0))
            .build();
    let csrf =
        axum_extra::extract::cookie::Cookie::build((config.auth.csrf_cookie_name.clone(), ""))
            .http_only(false)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .secure(config.auth.cookie_secure)
            .domain(config.server.cookie_domain.clone())
            .path("/")
            .max_age(TimeDuration::seconds(0))
            .build();
    jar.add(access).add(refresh).add(csrf)
}

fn to_user_response(value: &User) -> UserResponse {
    UserResponse {
        id: value.id,
        email: value.email.clone(),
        role: value.role,
        created_at: value.created_at,
    }
}
