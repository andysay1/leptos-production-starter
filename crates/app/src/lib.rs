use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_meta::*;

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

#[cfg(feature = "ssr")]
#[component]
pub fn PageShell(
    title: &'static str,
    options: LeptosOptions,
    client_scripts: bool,
    children: Children,
) -> impl IntoView {
    provide_meta_context();
    let options_for_scripts = options.clone();
    let options_for_reload = options;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Title text=title/>
                <Stylesheet id="leptos" href="/pkg/app.css"/>
                <Show when=move || client_scripts fallback=|| ()>
                    <HydrationScripts options=options_for_scripts.clone() islands=true/>
                    <AutoReload options=options_for_reload.clone()/>
                </Show>
                <MetaTags/>
            </head>
            <body>
                {children()}
            </body>
        </html>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
}

#[component]
pub fn LoginPage(flash_error: Option<String>) -> impl IntoView {
    let flash_error = flash_error.unwrap_or_default();
    let show_flash = !flash_error.is_empty();

    view! {
        <Show when=move || show_flash fallback=|| ()>
            <div class="max-w-md mx-auto mt-6 px-6">
                <div class="rounded-lg border border-rose-800/60 bg-rose-950/40 text-rose-200 px-4 py-3 text-sm">
                    {flash_error.clone()}
                </div>
            </div>
        </Show>
        <LoginFormIsland/>
    }
}

#[component]
pub fn RegisterPage(flash_error: Option<String>) -> impl IntoView {
    let flash_error = flash_error.unwrap_or_default();
    let show_flash = !flash_error.is_empty();

    view! {
        <Show when=move || show_flash fallback=|| ()>
            <div class="max-w-md mx-auto mt-6 px-6">
                <div class="rounded-lg border border-rose-800/60 bg-rose-950/40 text-rose-200 px-4 py-3 text-sm">
                    {flash_error.clone()}
                </div>
            </div>
        </Show>
        <RegisterFormIsland/>
    }
}

#[component]
pub fn DashboardPage(email: String) -> impl IntoView {
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
                <div class="card p-6 space-y-2">
                    <p class="text-sm text-slate-400">"Authenticated as"</p>
                    <p class="text-xl font-semibold">{email}</p>
                </div>
            </section>
        </main>
    }
}

#[island(lazy)]
pub fn LoginFormIsland() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let status = RwSignal::new(Option::<String>::None);

    let on_submit = move |_ev: leptos::ev::SubmitEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;

            _ev.prevent_default();
            status.set(Some("Submitting...".into()));

            let email_val = email.get();
            let password_val = password.get();
            let status = status.clone();
            spawn_local(async move {
                match login_api_request(email_val, password_val).await {
                    Ok(()) => {
                        if let Some(win) = web_sys::window() {
                            let _ = win.location().set_href("/app");
                        }
                    }
                    Err(msg) => status.set(Some(msg)),
                }
            });
        }
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center px-6">
            <div class="w-full max-w-md space-y-6">
                <div class="text-center space-y-2">
                    <p class="text-emerald-300 font-semibold text-sm uppercase tracking-widest">"Secure area"</p>
                    <h1 class="text-3xl font-bold">"Login"</h1>
                    <p class="text-slate-400 text-sm">"JWT + refresh cookie flow with CSRF token."</p>
                </div>
                <form class="card p-6 space-y-4" action="/app/login" method="post" on:submit=on_submit>
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
                        "Log in"
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

#[island(lazy)]
pub fn RegisterFormIsland() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let status = RwSignal::new(Option::<String>::None);

    let on_submit = move |_ev: leptos::ev::SubmitEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;

            _ev.prevent_default();
            status.set(Some("Submitting...".into()));

            let email_val = email.get();
            let password_val = password.get();
            let status = status.clone();
            spawn_local(async move {
                match register_api_request(email_val, password_val).await {
                    Ok(()) => {
                        if let Some(win) = web_sys::window() {
                            let _ = win.location().set_href("/app");
                        }
                    }
                    Err(msg) => status.set(Some(msg)),
                }
            });
        }
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center px-6">
            <div class="w-full max-w-md space-y-6">
                <div class="text-center space-y-2">
                    <p class="text-emerald-300 font-semibold text-sm uppercase tracking-widest">"Secure area"</p>
                    <h1 class="text-3xl font-bold">"Create account"</h1>
                    <p class="text-slate-400 text-sm">"JWT + refresh cookie flow with CSRF token."</p>
                </div>
                <form class="card p-6 space-y-4" action="/app/register" method="post" on:submit=on_submit>
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
                        "Create account"
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
#[leptos::prelude::lazy]
async fn login_api_request(email: String, password: String) -> Result<(), String> {
    let body = serde_json::json!({ "email": email, "password": password }).to_string();
    let txt = fetch_json("/api/auth/login", body)
        .await
        .map_err(|_| "Network error".to_string())?;
    let status = txt.0;
    let txt = txt.1;

    if status >= 200 && status < 400 {
        return Ok(());
    }
    let msg = serde_json::from_str::<serde_json::Value>(&txt)
        .ok()
        .and_then(|v| v.get("message")?.as_str().map(|s| s.to_string()))
        .unwrap_or(txt);
    Err(msg.trim().to_string())
}

#[cfg(target_arch = "wasm32")]
#[leptos::prelude::lazy]
async fn register_api_request(email: String, password: String) -> Result<(), String> {
    let body = serde_json::json!({ "email": email, "password": password }).to_string();
    let txt = fetch_json("/api/auth/register", body)
        .await
        .map_err(|_| "Network error".to_string())?;
    let status = txt.0;
    let txt = txt.1;

    if status >= 200 && status < 400 {
        return Ok(());
    }
    let msg = serde_json::from_str::<serde_json::Value>(&txt)
        .ok()
        .and_then(|v| v.get("message")?.as_str().map(|s| s.to_string()))
        .unwrap_or(txt);
    Err(msg.trim().to_string())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json(url: &str, body: String) -> Result<(u16, String), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    let Some(window) = web_sys::window() else {
        return Err(JsValue::from_str("missing window"));
    };

    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_credentials(web_sys::RequestCredentials::SameOrigin);
    init.set_body(&JsValue::from_str(&body));

    let request = web_sys::Request::new_with_str_and_init(url, &init)?;
    request.headers().set("Content-Type", "application/json")?;
    request.headers().set("Accept", "application/json")?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let status = resp.status() as u16;

    let text_promise = resp.text()?;
    let text = JsFuture::from(text_promise)
        .await?
        .as_string()
        .unwrap_or_default();

    Ok((status, text))
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
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
