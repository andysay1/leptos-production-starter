use axum::extract::FromRef;
use db::Database;
use domain::AuthService;
use leptos_config::LeptosOptions;
use metrics_exporter_prometheus::PrometheusHandle;
use redis::aio::ConnectionManager;
use shared::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub auth: AuthService<Database>,
    pub leptos_options: LeptosOptions,
    pub metrics: PrometheusHandle,
    pub redis: Option<ConnectionManager>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> LeptosOptions {
        state.leptos_options.clone()
    }
}
