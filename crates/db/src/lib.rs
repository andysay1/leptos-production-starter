use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::models::{NewUser, RefreshToken, User};
use domain::ports::{AuditLogRepository, RefreshTokenRepository, UserRepository};
use shared::config::DatabaseConfig;
use shared::error::{AppError, Result};
use shared::types::AuditEvent;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::instrument;
use uuid::Uuid;

pub type PgPool = Pool<Postgres>;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await
            .map_err(map_sqlx_error)?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::internal(e.to_string()))?;
        Ok(())
    }

    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}

#[async_trait]
impl UserRepository for Database {
    #[instrument(skip(self, new_user), fields(email = %new_user.email))]
    async fn create_user(&self, new_user: NewUser) -> Result<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, now(), now())
            RETURNING id, email, password_hash, role, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(new_user.email)
        .bind(new_user.password_hash)
        .bind(UserRoleDb::from(new_user.role))
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_users(&self, page: i64, per_page: i64) -> Result<(Vec<User>, i64)> {
        let offset = (page - 1).max(0) * per_page;
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total.0))
    }
}

#[async_trait]
impl RefreshTokenRepository for Database {
    async fn store_refresh_token(&self, token: &RefreshToken) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (token_hash) DO UPDATE SET expires_at = EXCLUDED.expires_at
            "#,
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn delete_refresh_token(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn delete_tokens_for_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}

#[async_trait]
impl AuditLogRepository for Database {
    async fn log_event(&self, event: AuditEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (id, user_id, event_type, ip, user_agent, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(event.id)
        .bind(event.user_id)
        .bind(event.event_type)
        .bind(event.ip)
        .bind(event.user_agent)
        .bind(event.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    role: UserRoleDb,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            role: row.role.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::Type, Copy, Clone, Debug, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
enum UserRoleDb {
    User,
    Admin,
}

impl From<UserRoleDb> for shared::types::UserRole {
    fn from(value: UserRoleDb) -> Self {
        match value {
            UserRoleDb::User => shared::types::UserRole::User,
            UserRoleDb::Admin => shared::types::UserRole::Admin,
        }
    }
}

impl From<shared::types::UserRole> for UserRoleDb {
    fn from(value: shared::types::UserRole) -> Self {
        match value {
            shared::types::UserRole::User => UserRoleDb::User,
            shared::types::UserRole::Admin => UserRoleDb::Admin,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RefreshTokenRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<RefreshTokenRow> for RefreshToken {
    fn from(row: RefreshTokenRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::RowNotFound => AppError::NotFound,
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23505") {
                AppError::Conflict("duplicate record".into())
            } else {
                AppError::internal(db_err.to_string())
            }
        }
        _ => AppError::internal(err.to_string()),
    }
}
