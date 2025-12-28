use crate::models::{AuditEventBuilder, NewUser, RefreshToken, User};
use crate::ports::AuthRepo;
use base64::Engine;
use chrono::{Duration, Utc};
use rand::RngCore;
use shared::dto::{LoginRequest, RegisterRequest};
use shared::error::{AppError, Result};
use shared::types::UserRole;
use std::sync::Arc;
use validator::Validate;

#[derive(Clone)]
pub struct AuthService<R>
where
    R: AuthRepo,
{
    repo: Arc<R>,
}

impl<R> AuthService<R>
where
    R: AuthRepo,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn register(&self, input: RegisterRequest, role: Option<UserRole>) -> Result<User> {
        input.validate()?;

        if let Some(existing) = self.repo.find_by_email(&input.email).await? {
            return Err(AppError::Conflict(format!(
                "user with email {} already exists",
                existing.email
            )));
        }

        let password_hash = crate::models::PasswordService::hash(&input.password)?;
        let new_user = NewUser {
            email: input.email,
            password_hash,
            role: role.unwrap_or_default(),
        };

        let user = self.repo.create_user(new_user).await?;
        let event = AuditEventBuilder::new("auth.register")
            .user_id(Some(user.id))
            .build();
        let _ = self.repo.log_event(event).await;

        Ok(user)
    }

    pub async fn login(&self, input: LoginRequest) -> Result<User> {
        input.validate()?;

        let user = self
            .repo
            .find_by_email(&input.email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !crate::models::PasswordService::verify(&user.password_hash, &input.password)? {
            return Err(AppError::Unauthorized);
        }

        let event = AuditEventBuilder::new("auth.login")
            .user_id(Some(user.id))
            .build();
        let _ = self.repo.log_event(event).await;

        Ok(user)
    }

    pub async fn store_refresh_token(
        &self,
        user_id: uuid::Uuid,
        refresh_token: &str,
        ttl_days: i64,
    ) -> Result<RefreshToken> {
        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(ttl_days))
            .ok_or_else(|| AppError::Internal("failed to compute refresh token expiry".into()))?;
        let token = RefreshToken::from_raw(user_id, refresh_token, expires_at);
        self.repo.store_refresh_token(&token).await?;
        Ok(token)
    }

    pub async fn validate_refresh_token(&self, raw_token: &str) -> Result<RefreshToken> {
        let token_hash = RefreshToken::hash(raw_token);
        let token = self
            .repo
            .find_refresh_token(&token_hash)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if token.expires_at < Utc::now() {
            let _ = self.repo.delete_refresh_token(token.id).await;
            return Err(AppError::Unauthorized);
        }

        Ok(token)
    }

    pub async fn revoke_all(&self, user_id: uuid::Uuid) -> Result<()> {
        self.repo.delete_tokens_for_user(user_id).await
    }

    pub async fn logout(&self, raw_token: &str) -> Result<()> {
        let token_hash = RefreshToken::hash(raw_token);
        if let Some(token) = self.repo.find_refresh_token(&token_hash).await? {
            let _ = self.repo.delete_refresh_token(token.id).await?;
        }
        Ok(())
    }
}

pub fn generate_refresh_token() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}
