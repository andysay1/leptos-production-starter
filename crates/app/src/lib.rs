#[cfg(feature = "hydrate")]
use leptos::mount::hydrate_body;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
#[cfg(target_arch = "wasm32")]
use leptos_router::hooks::use_navigate;
#[cfg(target_arch = "wasm32")]
use leptos_router::NavigateOptions;
use leptos_router::path;
#[cfg(target_arch = "wasm32")]
use shared::dto::TokenResponse;
use shared::dto::UserResponse;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                <Route path=path!("") view=LandingPage/>
                <Route path=path!("login") view=LoginPage/>
                <Route path=path!("register") view=RegisterPage/>
                <Route path=path!("app") view=PrivateApp/>
            </Routes>
        </Router>
    }
}

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-gradient-to-b from-slate-950 to-slate-900 text-slate-100">
            <section class="max-w-6xl mx-auto px-6 pt-16 pb-12">
                <header class="flex flex-col md:flex-row md:items-center md:justify-between gap-10">
                    <div class="space-y-6 max-w-2xl">
                        <p class="px-4 py-1 rounded-full bg-emerald-500/10 text-emerald-300 inline-flex items-center gap-2 w-fit text-sm font-semibold">"Production-ready Leptos foundation"</p>
                        <h1 class="text-4xl md:text-5xl font-extrabold leading-tight">
                            "SSR + API + Auth skeleton for shipping Rust web products fast."
                        </h1>
                        <p class="text-lg text-slate-300 max-w-2xl">
                            "Strict architecture, typed APIs, JWT/refresh auth, observability hooks, and CI/CD baked in. Copy, configure, and ship."
                        </p>
                        <div class="flex flex-wrap gap-4">
                            <a href="/app/register" class="btn-primary">"Create account"</a>
                            <a href="/app/login" class="btn-secondary border border-slate-700 px-4 py-2 rounded-lg">"Log in"</a>
                        </div>
                        <div class="flex gap-4 text-sm text-slate-400">
                            <span>"SSR + Hydration"</span>
                            <span>"|"</span>
                            <span>"PostgreSQL + SQLx migrations"</span>
                            <span>"|"</span>
                            <span>"Tracing + metrics"</span>
                        </div>
                    </div>
   
                </header>
            </section>
        </main>
    }
}

#[component]
pub fn SpaApp() -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                <Route path=path!("app") view=PrivateApp/>
                <Route path=path!("app/login") view=LoginPage/>
                <Route path=path!("app/register") view=RegisterPage/>
                <Route path=path!("app/logout") view=LogoutPage/>
                <Route path=path!("app/*any") view=NotFound/>
            </Routes>
        </Router>
    }
}

#[cfg(feature = "ssr")]
#[component]
pub fn SpaShell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Title text="Leptos Production Starter"/>
                <Stylesheet id="leptos" href="/pkg/app.css"/>
                <HydrationScripts options=options.clone()/>
                <AutoReload options=options/>
                <MetaTags/>
            </head>
            <body>
                <SpaApp/>
            </body>
        </html>
    }
}

#[component]
fn Metric(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-lg bg-slate-900/60 border border-slate-800 p-3">
            <p class="text-slate-400 text-xs uppercase tracking-wide">{label}</p>
            <p class="text-xl font-semibold text-emerald-300">{value}</p>
        </div>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    hydrate_body(|| view! { <SpaApp/> });
}

#[component]
fn LoginPage() -> impl IntoView {
    view! {
        <AuthForm title="Login" _api_action="/api/auth/login" form_action="/app/login" kind=AuthFormKind::Login/>
    }
}

#[component]
fn RegisterPage() -> impl IntoView {
    view! {
        <AuthForm title="Create account" _api_action="/api/auth/register" form_action="/app/register" kind=AuthFormKind::Register/>
    }
}

#[derive(Clone, Copy)]
enum AuthFormKind {
    Login,
    Register,
}

#[component]
fn AuthForm(
    title: &'static str,
    _api_action: &'static str,
    form_action: &'static str,
    kind: AuthFormKind,
) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let status = RwSignal::new(Option::<String>::None);
    #[cfg(target_arch = "wasm32")]
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        status.set(Some("Submitting...".into()));
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::logging::log;
            use wasm_bindgen_futures::spawn_local;

            let email_val = email.get();
            let password_val = password.get();
            let status = status.clone();
            let navigate = navigate.clone();
            let action_url = _api_action;
            spawn_local(async move {
                let body = serde_json::json!({ "email": email_val, "password": password_val });
                let response = reqwasm::http::Request::post(action_url)
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.status() == 200 || resp.status() == 201 => {
                        if let Ok(token) = resp.json::<TokenResponse>().await {
                            let _ = store_tokens(&token);
                            status.set(Some("Success! Redirecting...".into()));
                            navigate("/app", NavigateOptions::default());
                        } else {
                            status.set(Some("Auth response parse error".into()));
                        }
                    }
                    Ok(resp) => {
                        let msg = resp.text().await.unwrap_or_else(|_| "Auth failed".into());
                        status.set(Some(msg));
                    }
                    Err(err) => {
                        log!("auth error: {err:?}");
                        status.set(Some("Network error".into()));
                    }
                }
            });
        }
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center px-6">
            <div class="w-full max-w-md space-y-6">
                <div class="text-center space-y-2">
                    <p class="text-emerald-300 font-semibold text-sm uppercase tracking-widest">"Secure area"</p>
                    <h1 class="text-3xl font-bold">{title}</h1>
                    <p class="text-slate-400 text-sm">"JWT + refresh cookie flow with CSRF token."</p>
                </div>
                <form class="card p-6 space-y-4" action=form_action method="post" on:submit=on_submit>
                    <label class="block space-y-2">
                        <span class="text-sm text-slate-300">"Email"</span>
                        <input
                            class="input"
                            type="email"
                            name="email"
                            placeholder="you@example.com"
                            on:input=move |ev| email.set(event_target_value(&ev))
                            required
                        />
                    </label>
                    <label class="block space-y-2">
                        <span class="text-sm text-slate-300">"Password"</span>
                        <input
                            class="input"
                            type="password"
                            name="password"
                            placeholder="••••••••"
                            on:input=move |ev| password.set(event_target_value(&ev))
                            required
                        />
                    </label>
                    <button type="submit" class="btn-primary w-full">
                        {match kind { AuthFormKind::Login => "Log in", AuthFormKind::Register => "Create account" }}
                    </button>
                    <Show when=move || status.get().is_some() fallback=|| ()>
                        <p class="text-sm text-slate-300 bg-slate-900/60 px-3 py-2 rounded">
                            {move || status.get().unwrap_or_default()}
                        </p>
                    </Show>
                </form>
                <div class="text-center text-sm text-slate-400">
                    <a href="/" rel="external" class="text-emerald-300 hover:text-emerald-200">"Back to landing"</a>
                </div>
            </div>
        </main>
    }
}

#[cfg(target_arch = "wasm32")]
fn store_tokens(token: &TokenResponse) -> Result<(), String> {
    use web_sys::window;
    if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.set_item("access_token", &token.access_token);
        let _ = storage.set_item("csrf_token", &token.csrf_token);
    }
    Ok(())
}

#[component]
pub fn PrivateApp() -> impl IntoView {
    let me = LocalResource::new(|| async move {
            #[cfg(target_arch = "wasm32")]
            {
                let access = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                    .and_then(|s| s.get_item("access_token").ok())
                    .flatten();
                let request = reqwasm::http::Request::get("/api/me");
                let request = if let Some(token) = access {
                    request.header("Authorization", &format!("Bearer {token}"))
                } else {
                    request
                };
                let resp = request.send().await;
                if let Ok(resp) = resp {
                    if resp.status() == 200 {
                        return resp.json::<UserResponse>().await.ok();
                    }
                }
            }
            None
        });

    #[cfg(target_arch = "wasm32")]
    {
        // Trigger refetch after hydration to pick up stored tokens.
        Effect::new(move |_| {
            me.refetch();
        });
    }

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100">
            <section class="max-w-5xl mx-auto px-6 py-12 space-y-6">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm text-emerald-300 uppercase tracking-widest">"Private area"</p>
                        <h1 class="text-3xl font-bold">"App dashboard"</h1>
                    </div>
                    <a href="/logout" rel="external" class="text-sm text-slate-400 hover:text-slate-200">"Logout"</a>
                </div>
                <Suspense fallback=move || view! { <div class="card p-6"><p class="text-slate-300">"Loading session..."</p></div> }>
                    {move || {
                        match me.get() {
                            Some(Some(user)) => render_user(user).into_any(),
                            Some(None) => view! {
                                <div class="card p-6 space-y-3">
                                    <p class="text-slate-300">"You are not authenticated."</p>
                                    <div class="flex gap-3">
                                        <A href="/app/login" {..} class="btn-primary">"Log in"</A>
                                        <A href="/app/register" {..} class="btn-secondary border border-slate-800 px-4 py-2 rounded-lg">"Register"</A>
                                    </div>
                                </div>
                            }.into_any(),
                            None => view! { <p class="text-slate-400">"Loading user..."</p> }.into_any(),
                        }
                    }}
                </Suspense>
            </section>
        </main>
    }
}

#[component]
fn LogoutPage() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(async move {
            let csrf = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|s| s.get_item("csrf_token").ok())
                .flatten();

            let mut req = reqwasm::http::Request::post("/api/auth/logout");
            if let Some(csrf) = csrf {
                req = req.header("x-csrf-token", &csrf);
            }

            let _ = req.send().await;

            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item("access_token");
                let _ = storage.remove_item("csrf_token");
            }

            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        });
    }

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center px-6">
            <div class="card p-6">
                <p class="text-slate-300">"Logging out..."</p>
            </div>
        </main>
    }
}

fn render_user(user: UserResponse) -> AnyView {
    view! {
        <div class="card p-6 space-y-2">
            <p class="text-sm text-slate-400">"Authenticated as"</p>
            <p class="text-xl font-semibold">{user.email.clone()}</p>
            <p class="text-sm text-slate-400">{"Role: "}{format!("{:?}", user.role)}</p>
        </div>
    }
    .into_any()
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(res) = use_context::<leptos_axum::ResponseOptions>() {
            res.set_status(http::StatusCode::NOT_FOUND);
        }
    }

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center">
            <div class="text-center space-y-4">
                <p class="text-sm text-emerald-300 uppercase tracking-widest">"404"</p>
                <h1 class="text-3xl font-bold">"Page not found"</h1>
                <a href="/" rel="external" class="btn-primary">"Back home"</a>
            </div>
        </main>
    }
}
