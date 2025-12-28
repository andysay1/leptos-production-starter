use leptos::*;
use leptos_meta::*;
use leptos_router::*;
#[cfg(target_arch = "wasm32")]
use shared::dto::TokenResponse;
use shared::dto::UserResponse;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/app.css"/>
        <Title text="Leptos Production Starter"/>
        <Router>
            <Routes>
                <Route path="" view=LandingPage/>
                <Route path="/login" view=LoginPage/>
                <Route path="/register" view=RegisterPage/>
                <Route path="/app" view=PrivateApp/>
                <Route path="/*any" view=NotFound/>
            </Routes>
        </Router>
    }
}

#[component]
fn LandingPage() -> impl IntoView {
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
                            <A href="/register" class="btn-primary">"Create account"</A>
                            <A href="/login" class="btn-secondary border border-slate-700 px-4 py-2 rounded-lg">"Log in"</A>
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
    leptos::mount_to_body(|| leptos::view! { <App/> });
}

#[component]
fn LoginPage() -> impl IntoView {
    view! {
        <AuthForm title="Login" _action="/api/auth/login" kind=AuthFormKind::Login/>
    }
}

#[component]
fn RegisterPage() -> impl IntoView {
    view! {
        <AuthForm title="Create account" _action="/api/auth/register" kind=AuthFormKind::Register/>
    }
}

#[derive(Clone, Copy)]
enum AuthFormKind {
    Login,
    Register,
}

#[component]
fn AuthForm(title: &'static str, _action: &'static str, kind: AuthFormKind) -> impl IntoView {
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let status = create_rw_signal(Option::<String>::None);
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
            let action_url = _action;
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
                            navigate("/app", Default::default());
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
                <form class="card p-6 space-y-4" on:submit=on_submit>
                    <label class="block space-y-2">
                        <span class="text-sm text-slate-300">"Email"</span>
                        <input
                            class="input"
                            type="email"
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
                    <A class="text-emerald-300 hover:text-emerald-200" href="/">"Back to landing"</A>
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
    let me: Resource<(), Option<UserResponse>> = create_resource(
        || (),
        |_| async move {
            #[cfg(target_arch = "wasm32")]
            {
                let access = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                    .and_then(|s| s.get_item("access_token").ok())
                    .flatten();
                if let Some(token) = access {
                    let resp = reqwasm::http::Request::get("/api/me")
                        .header("Authorization", &format!("Bearer {token}"))
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if resp.status() == 200 {
                            return resp.json::<UserResponse>().await.ok();
                        }
                    }
                }
            }
            None
        },
    );

    #[cfg(target_arch = "wasm32")]
    {
        use leptos::create_effect;
        // Trigger refetch after hydration to pick up stored tokens.
        create_effect(move |_| {
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
                    <A href="/logout" class="text-sm text-slate-400 hover:text-slate-200">"Logout via API"</A>
                </div>
                <Suspense fallback=move || view! { <div class="card p-6"><p class="text-slate-300">"Loading session..."</p></div> }>
                    {move || {
                        match me.get() {
                            Some(Some(user)) => render_user(user),
                            Some(None) => view! {
                                <div class="card p-6 space-y-3">
                                    <p class="text-slate-300">"You are not authenticated."</p>
                                    <div class="flex gap-3">
                                        <A class="btn-primary" href="/login">"Log in"</A>
                                        <A class="btn-secondary border border-slate-800 px-4 py-2 rounded-lg" href="/register">"Register"</A>
                                    </div>
                                </div>
                            }.into_view(),
                            None => view! { <p class="text-slate-400">"Loading user..."</p> }.into_view(),
                        }
                    }}
                </Suspense>
            </section>
        </main>
    }
}

fn render_user(user: UserResponse) -> View {
    view! {
        <div class="card p-6 space-y-2">
            <p class="text-sm text-slate-400">"Authenticated as"</p>
            <p class="text-xl font-semibold">{user.email.clone()}</p>
            <p class="text-sm text-slate-400">{"Role: "}{format!("{:?}", user.role)}</p>
        </div>
    }.into_view()
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center">
            <div class="text-center space-y-4">
                <p class="text-sm text-emerald-300 uppercase tracking-widest">"404"</p>
                <h1 class="text-3xl font-bold">"Page not found"</h1>
                <A class="btn-primary" href="/">"Back home"</A>
            </div>
        </main>
    }
}
