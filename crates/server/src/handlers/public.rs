use axum::response::{Html, IntoResponse};
use http::header;

pub async fn landing() -> impl IntoResponse {
    let html = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Leptos Production Starter</title>
    <link rel="stylesheet" href="/pkg/app.css">
  </head>
  <body>
    <main class="min-h-screen bg-gradient-to-b from-slate-950 to-slate-900 text-slate-100">
      <section class="max-w-6xl mx-auto px-6 pt-16 pb-12">
        <header class="flex flex-col md:flex-row md:items-center md:justify-between gap-10">
          <div class="space-y-6 max-w-2xl">
            <p class="px-4 py-1 rounded-full bg-emerald-500/10 text-emerald-300 inline-flex items-center gap-2 w-fit text-sm font-semibold">Production-ready Leptos foundation</p>
            <h1 class="text-4xl md:text-5xl font-extrabold leading-tight">SSR + API + Auth skeleton for shipping Rust web products fast.</h1>
            <p class="text-lg text-slate-300 max-w-2xl">Strict architecture, typed APIs, JWT/refresh auth, observability hooks, and CI/CD baked in. Copy, configure, and ship.</p>
            <div class="flex flex-wrap gap-4">
              <a href="/app/register" class="btn-primary">Create account</a>
              <a href="/app/login" class="btn-secondary border border-slate-700 px-4 py-2 rounded-lg">Log in</a>
            </div>
            <div class="flex gap-4 text-sm text-slate-400">
              <span>SSR + Hydration</span><span>|</span><span>PostgreSQL + SQLx migrations</span><span>|</span><span>Tracing + metrics</span>
            </div>
          </div>
        </header>
      </section>
    </main>
  </body>
</html>"#
        .to_string();

    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(html),
    )
}
