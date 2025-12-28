use crate::error::AppError;
use crate::types::Environment;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct AppConfig {
    #[validate]
    #[serde(default)]
    pub server: ServerConfig,
    #[validate]
    #[serde(default)]
    pub database: DatabaseConfig,
    #[validate]
    #[serde(default)]
    pub auth: AuthConfig,
    #[validate]
    #[serde(default)]
    pub tracing: TracingConfig,
    #[validate]
    #[serde(default)]
    pub redis: Option<RedisConfig>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        let mut builder = config::Config::builder().add_source(
            config::Environment::default()
                .separator("__")
                .try_parsing(true)
                .list_separator(","),
        );

        if let Ok(url) = std::env::var("DATABASE_URL") {
            builder = builder
                .set_override("database.url", url)
                .map_err(|e| AppError::config(format!("failed to set database url: {e}")))?;
        }

        if let Ok(url) = std::env::var("REDIS_URL") {
            builder = builder
                .set_override("redis.url", url)
                .map_err(|e| AppError::config(format!("failed to set redis url: {e}")))?;
        }

        if let Ok(secret) = std::env::var("JWT_SECRET") {
            builder = builder
                .set_override("auth.jwt_secret", secret)
                .map_err(|e| AppError::config(format!("failed to set jwt secret: {e}")))?;
        }

        if let Ok(secret) = std::env::var("REFRESH_TOKEN_SECRET") {
            builder = builder
                .set_override("auth.refresh_secret", secret)
                .map_err(|e| AppError::config(format!("failed to set refresh secret: {e}")))?;
        }

        if let Ok(secret) = std::env::var("CSRF_SECRET") {
            builder = builder
                .set_override("auth.csrf_secret", secret)
                .map_err(|e| AppError::config(format!("failed to set csrf secret: {e}")))?;
        }

        if let Ok(level) = std::env::var("RUST_LOG") {
            builder = builder
                .set_override("tracing.log_level", level)
                .map_err(|e| AppError::config(format!("failed to set log level: {e}")))?;
        }

        if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            builder = builder
                .set_override("tracing.otel_endpoint", endpoint)
                .map_err(|e| AppError::config(format!("failed to set otel endpoint: {e}")))?;
        }

        if let Ok(host) = std::env::var("APP_HOST") {
            builder = builder
                .set_override("server.host", host)
                .map_err(|e| AppError::config(format!("failed to set server host: {e}")))?;
        }

        if let Ok(port) = std::env::var("APP_PORT") {
            builder = builder
                .set_override("server.port", port)
                .map_err(|e| AppError::config(format!("failed to set server port: {e}")))?;
        }

        if let Ok(base_url) = std::env::var("APP_BASE_URL") {
            builder = builder
                .set_override("server.base_url", base_url)
                .map_err(|e| AppError::config(format!("failed to set base url: {e}")))?;
        }

        if let Ok(domain) = std::env::var("COOKIE_DOMAIN") {
            builder = builder
                .set_override("server.cookie_domain", domain)
                .map_err(|e| AppError::config(format!("failed to set cookie domain: {e}")))?;
        }

        if let Ok(env) = std::env::var("APP_ENV") {
            builder = builder
                .set_override("server.env", env)
                .map_err(|e| AppError::config(format!("failed to set app env: {e}")))?;
        }

        let config = builder
            .build()
            .map_err(|e| AppError::config(format!("failed to read configuration: {e}")))?;

        let cfg: AppConfig = config
            .try_deserialize()
            .map_err(|e| AppError::config(format!("invalid configuration: {e}")))?;

        cfg.validate()
            .map_err(|e| AppError::config(format!("configuration validation failed: {e}")))?;

        Ok(cfg)
    }

    pub fn addr(&self) -> Result<SocketAddr, AppError> {
        let host = self.server.host.parse().map_err(|e| {
            AppError::config(format!("invalid server host {}: {e}", self.server.host))
        })?;
        Ok(SocketAddr::new(host, self.server.port))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ServerConfig {
    #[serde(default)]
    pub env: Environment,
    #[validate(length(min = 3))]
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[validate(url)]
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[validate(length(min = 1))]
    #[serde(default = "default_cookie_domain")]
    pub cookie_domain: String,
    #[validate(length(min = 8))]
    #[serde(default = "default_app_name")]
    pub app_name: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            env: Environment::Development,
            host: default_host(),
            port: default_port(),
            base_url: default_base_url(),
            cookie_domain: default_cookie_domain(),
            app_name: default_app_name(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct DatabaseConfig {
    #[validate(length(min = 1))]
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
    #[serde(default = "default_min_pool_size")]
    pub min_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: default_pool_size(),
            min_connections: default_min_pool_size(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct RedisConfig {
    #[validate(length(min = 1))]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct AuthConfig {
    #[validate(length(min = 32))]
    pub jwt_secret: String,
    #[validate(length(min = 32))]
    pub refresh_secret: String,
    #[validate(length(min = 16))]
    pub csrf_secret: String,
    #[serde(default = "default_access_ttl")]
    pub access_token_ttl_minutes: u64,
    #[serde(default = "default_refresh_ttl")]
    pub refresh_token_ttl_days: u64,
    #[validate(length(min = 1))]
    #[serde(default = "default_access_cookie")]
    pub access_cookie_name: String,
    #[validate(length(min = 1))]
    #[serde(default = "default_refresh_cookie")]
    pub refresh_cookie_name: String,
    #[validate(length(min = 1))]
    #[serde(default = "default_csrf_cookie")]
    pub csrf_cookie_name: String,
    #[serde(default)]
    pub cookie_secure: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            refresh_secret: String::new(),
            csrf_secret: String::new(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 14,
            access_cookie_name: "access_token".into(),
            refresh_cookie_name: "refresh_token".into(),
            csrf_cookie_name: "csrf_token".into(),
            cookie_secure: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct TracingConfig {
    #[validate(length(min = 1))]
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub otel_endpoint: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            otel_endpoint: None,
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    3000
}

fn default_base_url() -> String {
    "http://localhost:3000".into()
}

fn default_cookie_domain() -> String {
    "localhost".into()
}

fn default_app_name() -> String {
    "leptos-app".into()
}

fn default_pool_size() -> u32 {
    10
}

fn default_min_pool_size() -> u32 {
    2
}

fn default_access_ttl() -> u64 {
    15
}

fn default_refresh_ttl() -> u64 {
    14
}

fn default_access_cookie() -> String {
    "access_token".into()
}

fn default_refresh_cookie() -> String {
    "refresh_token".into()
}

fn default_csrf_cookie() -> String {
    "csrf_token".into()
}

fn default_log_level() -> String {
    "info".into()
}
