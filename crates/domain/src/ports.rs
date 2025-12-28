use crate::models::{NewUser, RefreshToken, User};
use async_trait::async_trait;
use shared::error::Result;
use shared::types::AuditEvent;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> Result<User>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn list_users(&self, page: i64, per_page: i64) -> Result<(Vec<User>, i64)>;
}

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn store_refresh_token(&self, token: &RefreshToken) -> Result<()>;
    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>>;
    async fn delete_refresh_token(&self, id: Uuid) -> Result<()>;
    async fn delete_tokens_for_user(&self, user_id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn log_event(&self, event: AuditEvent) -> Result<()>;
}

pub trait AuthRepo:
    UserRepository + RefreshTokenRepository + AuditLogRepository + Send + Sync + 'static
{
}

impl<T> AuthRepo for T where
    T: UserRepository + RefreshTokenRepository + AuditLogRepository + Send + Sync + 'static
{
}
