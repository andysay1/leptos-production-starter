mod handlers;
mod security;
mod state;
mod telemetry;

use crate::handlers::{auth, health, users};
use crate::handlers::public;
use crate::state::AppState;
use anyhow::Context;
use axum::{
    http,
    http::{HeaderValue, StatusCode},
    response::Redirect,
    middleware,
    routing::{get, post},
    Router,
};
use leptos_axum::{generate_route_list, LeptosRoutes};
use metrics_exporter_prometheus::PrometheusHandle;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::future::IntoFuture;
use tokio::task::LocalSet;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = shared::config::AppConfig::from_env()?;
    telemetry::init_tracing(&config.tracing)?;

    let leptos_config = leptos_config::get_configuration(None)?;
    let mut leptos_options = leptos_config.leptos_options;

    let addr: SocketAddr = config.addr().context("invalid server addr")?;
    leptos_options.site_addr = addr;

    let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install prometheus recorder");

    let db = db::Database::connect(&config.database).await?;
    db.migrate().await?;

    let redis = match &config.redis {
        Some(redis_cfg) => match redis::Client::open(redis_cfg.url.clone()) {
            Ok(client) => client.get_tokio_connection_manager().await.ok(),
            Err(_) => None,
        },
        None => None,
    };

    let auth = domain::AuthService::new(Arc::new(db.clone()));

    let state = AppState {
        config: config.clone(),
        db: db.clone(),
        auth,
        leptos_options: leptos_options.clone(),
        metrics: metrics_handle.clone(),
        redis,
    };

    let app = build_router(state.clone(), leptos_options, metrics_handle);

    info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Leptos uses !Send futures in some cases; wrap server in LocalSet to allow spawn_local.
    let local = LocalSet::new();
    local
        .run_until(async move {
            axum::serve(listener, app).into_future().await?;
            Ok::<(), anyhow::Error>(())
        })
        .await?;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

fn build_router(
    state: AppState,
    leptos_options: leptos_config::LeptosOptions,
    metrics_handle: PrometheusHandle,
) -> Router<()> {
    let site_root: PathBuf = PathBuf::from(leptos_options.site_root.as_ref());
    let pkg_dir = site_root.join("pkg");
    let leptos_routes = generate_route_list(app::SpaApp);
    let shell_options = state.leptos_options.clone();

    let auth_routes = Router::<AppState>::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/refresh", post(auth::refresh))
        .route("/api/me", get(auth::me));

    let api_routes = Router::<AppState>::new()
        .route("/api/health", get(health::health))
        .route("/api/ready", get(health::ready))
        .route("/api/users", get(users::list_users))
        .merge(auth_routes);

    let trace_layer = TraceLayer::new_for_http();

    let metrics_route = Router::<AppState>::new().route(
        "/metrics",
        get({
            move || {
                let handle = metrics_handle.clone();
                async move {
                    (
                        StatusCode::OK,
                        [(
                            http::header::CONTENT_TYPE,
                            HeaderValue::from_static("text/plain; version=0.0.4"),
                        )],
                        handle.render(),
                    )
                }
            }
        }),
    );

    let router = Router::<AppState>::new()
        .route("/", get(public::landing))
        .route("/login", get(|| async { Redirect::temporary("/app/login") }))
        .route("/register", get(|| async { Redirect::temporary("/app/register") }))
        .route("/logout", get(auth::logout_get))
        .route("/app/login", post(auth::login_form))
        .route("/app/register", post(auth::register_form))
        .merge(api_routes)
        .merge(metrics_route)
        .leptos_routes(&state, leptos_routes, move || {
            leptos::prelude::view! { <app::SpaShell options=shell_options.clone()/> }
        });

    router
        .nest_service(
            "/pkg",
            ServeDir::new(pkg_dir)
                .precompressed_br()
                .precompressed_gzip(),
        )
        .nest_service(
            "/assets",
            ServeDir::new(&site_root)
                .precompressed_br()
                .precompressed_gzip(),
        )
        .layer(middleware::from_fn(inject_request_id))
        .layer(trace_layer)
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback(public::not_found)
}

async fn inject_request_id(
    mut req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(request_id.clone());

    let mut res = next.run(req).await;
    if !res.headers().contains_key("x-request-id") {
        if let Ok(value) = HeaderValue::from_str(&request_id) {
            res.headers_mut()
                .insert(http::header::HeaderName::from_static("x-request-id"), value);
        }
    }
    res
}
