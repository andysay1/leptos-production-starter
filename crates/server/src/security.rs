use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::Engine;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use shared::config::AppConfig;
use shared::error::{AppError, Result};
use shared::types::Claims;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::Duration as TimeDuration;

pub fn sign_access_token(
    user_id: uuid::Uuid,
    role: shared::types::UserRole,
    cfg: &shared::config::AuthConfig,
) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::internal(format!("clock error: {e}")))?;
    let exp = now + Duration::from_secs(cfg.access_token_ttl_minutes * 60);

    let claims = Claims {
        sub: user_id,
        role,
        exp: exp.as_secs() as usize,
        iat: now.as_secs() as usize,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::internal(format!("token signing error: {e}")))
}

pub fn decode_access_token(token: &str, cfg: &shared::config::AuthConfig) -> Result<Claims> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub fn build_refresh_cookie(raw: &str, config: &AppConfig) -> Cookie<'static> {
    let mut cookie = Cookie::build((config.auth.refresh_cookie_name.clone(), raw.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(config.auth.cookie_secure)
        .domain(config.server.cookie_domain.clone());
    cookie = cookie.max_age(TimeDuration::days(
        config.auth.refresh_token_ttl_days as i64,
    ));
    cookie.build()
}

pub fn build_csrf_cookie(token: &str, config: &AppConfig) -> Cookie<'static> {
    Cookie::build((config.auth.csrf_cookie_name.clone(), token.to_string()))
        .http_only(false)
        .same_site(SameSite::Lax)
        .secure(config.auth.cookie_secure)
        .path("/")
        .domain(config.server.cookie_domain.clone())
        .build()
}

pub fn build_access_cookie(token: &str, config: &AppConfig) -> Cookie<'static> {
    let mut cookie =
        Cookie::build((config.auth.access_cookie_name.clone(), token.to_string()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(config.auth.cookie_secure)
            .path("/")
            .domain(config.server.cookie_domain.clone());
    cookie = cookie.max_age(TimeDuration::seconds(
        (config.auth.access_token_ttl_minutes * 60) as i64,
    ));
    cookie.build()
}

pub fn generate_csrf_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn verify_csrf(csrf_header: Option<&str>, csrf_cookie: Option<&str>) -> Result<()> {
    match (csrf_header, csrf_cookie) {
        (Some(header), Some(cookie)) if header == cookie => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

pub fn bearer_token(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}
