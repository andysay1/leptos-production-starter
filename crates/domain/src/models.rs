use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::error::{AppError, Result};
use shared::types::{AuditEvent, UserRole};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn from_raw(user_id: Uuid, raw: &str, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash: Self::hash(raw),
            expires_at,
            created_at: Utc::now(),
        }
    }

    pub fn hash(raw: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    AuthLogin,
    AuthRegister,
    AuthLogout,
    TokenRefresh,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::AuthLogin => "auth.login",
            AuditEventType::AuthRegister => "auth.register",
            AuditEventType::AuthLogout => "auth.logout",
            AuditEventType::TokenRefresh => "auth.refresh",
        }
    }
}

pub struct AuditEventBuilder {
    event_type: String,
    user_id: Option<Uuid>,
    ip: Option<String>,
    user_agent: Option<String>,
}

impl AuditEventBuilder {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            user_id: None,
            ip: None,
            user_agent: None,
        }
    }

    pub fn user_id(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn ip(mut self, ip: Option<String>) -> Self {
        self.ip = ip;
        self
    }

    pub fn user_agent(mut self, ua: Option<String>) -> Self {
        self.user_agent = ua;
        self
    }

    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: Uuid::new_v4(),
            user_id: self.user_id,
            event_type: self.event_type,
            ip: self.ip,
            user_agent: self.user_agent,
            created_at: Utc::now(),
        }
    }
}

pub struct PasswordService;

impl PasswordService {
    pub fn hash(password: &str) -> Result<String> {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let salt = SaltString::encode_b64(&salt)
            .map_err(|e| AppError::internal(format!("failed to encode salt: {e}")))?;
        argon2::Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|p| p.to_string())
            .map_err(|e| AppError::internal(format!("failed to hash password: {e}")))
    }

    pub fn verify(hash: &str, candidate: &str) -> Result<bool> {
        let parsed = PasswordHash::new(hash).map_err(|_| AppError::Unauthorized)?;
        Ok(argon2::Argon2::default()
            .verify_password(candidate.as_bytes(), &parsed)
            .is_ok())
    }
}
