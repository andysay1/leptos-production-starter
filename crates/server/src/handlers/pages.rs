use crate::handlers::RequestIdExtractor;
use crate::security;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use domain::ports::UserRepository;
use leptos::prelude::provide_context;

fn bearer_or_cookie_token(state: &AppState, headers: &HeaderMap, jar: &CookieJar) -> Option<String> {
    security::bearer_token(headers).or_else(|| {
        jar.get(&state.config.auth.access_cookie_name)
            .map(|c| c.value().to_string())
    })
}

fn take_flash_error_cookie(state: &AppState, jar: CookieJar) -> (CookieJar, Option<String>) {
    let Some(cookie) = jar.get(security::FLASH_ERROR_COOKIE_NAME) else {
        return (jar, None);
    };

    let msg = security::decode_flash_error(cookie.value());
    let cleared =
        jar.add(security::build_flash_error_cookie("", &state.config, 0));
    (cleared, msg)
}

pub async fn app_login_page(
    State(state): State<AppState>,
    jar: CookieJar,
    req: Request<Body>,
) -> impl IntoResponse {
    let (jar, flash_error) = take_flash_error_cookie(&state, jar);
    let leptos_options = state.leptos_options.clone();
    let state_for_ctx = state.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(state_for_ctx.clone());
        },
        move || {
            let flash_error = flash_error.clone();
            leptos::prelude::view! {
                <app::PageShell title="Login" options=leptos_options.clone() client_scripts=true>
                    <app::LoginPage flash_error/>
                </app::PageShell>
            }
        },
    );

    (jar, handler(req).await)
}

pub async fn app_register_page(
    State(state): State<AppState>,
    jar: CookieJar,
    req: Request<Body>,
) -> impl IntoResponse {
    let (jar, flash_error) = take_flash_error_cookie(&state, jar);
    let leptos_options = state.leptos_options.clone();
    let state_for_ctx = state.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(state_for_ctx.clone());
        },
        move || {
            let flash_error = flash_error.clone();
            leptos::prelude::view! {
                <app::PageShell title="Register" options=leptos_options.clone() client_scripts=true>
                    <app::RegisterPage flash_error/>
                </app::PageShell>
            }
        },
    );

    (jar, handler(req).await)
}

pub async fn app_dashboard(
    State(state): State<AppState>,
    _request_id: RequestIdExtractor,
    headers: HeaderMap,
    jar: CookieJar,
    req: Request<Body>,
) -> Response {
    let Some(token) = bearer_or_cookie_token(&state, &headers, &jar) else {
        return Redirect::to("/app/login").into_response();
    };

    let claims = match security::decode_access_token(&token, &state.config.auth) {
        Ok(c) => c,
        Err(_) => return Redirect::to("/app/login").into_response(),
    };

    let user = match state.db.find_by_id(claims.sub).await {
        Ok(Some(user)) => user,
        _ => return Redirect::to("/app/login").into_response(),
    };

    let leptos_options = state.leptos_options.clone();
    let state_for_ctx = state.clone();
    let email = user.email.clone();

    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(state_for_ctx.clone());
        },
        move || {
            let email = email.clone();
            leptos::prelude::view! {
                <app::PageShell title="Dashboard" options=leptos_options.clone() client_scripts=false>
                    <app::DashboardPage email/>
                </app::PageShell>
            }
        },
    );

    handler(req).await
}

pub async fn app_not_found(State(state): State<AppState>, req: Request<Body>) -> Response {
    let leptos_options = state.leptos_options.clone();
    let state_for_ctx = state.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(state_for_ctx.clone());
        },
        move || {
            leptos::prelude::view! {
                <app::PageShell title="Not found" options=leptos_options.clone() client_scripts=false>
                    <app::NotFoundPage/>
                </app::PageShell>
            }
        },
    );

    handler(req).await
}
