# Leptos Production Starter

A production-ready Leptos/Axum template with SSR + hydration, auth, SQLx migrations, Redis, observability, Docker/CI, and a Tailwind-powered UI.

## Prerequisites
- Rust toolchain (MSRV 1.84.0) and `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Node.js (for Tailwind; optional if you rely on cargo-leptos to run Tailwind CLI)
- Docker + Docker Compose (for Postgres/Redis in dev)
- `cargo-leptos` and `wasm-bindgen` CLIs installed (binaries are already in `~/.cargo/bin` from this setup; install otherwise via the Leptos docs)

## Quick start (dev)
```bash
cp .env.example .env             # fill secrets: JWT/refresh/CSRF (lengths enforced)
docker compose -f infra/docker/compose.dev.yml up -d db redis
cargo leptos watch -- --bin server  # hot reload (SSR + front)
```
Open http://localhost:3000. Auth flow uses cookies (domain must match `COOKIE_DOMAIN` in `.env`).

## Useful commands
- `make dev`        — compose (db/redis) + cargo-leptos watch
- `make migrate`    — run SQLx migrations
- `make seed`       — seed data via CLI
- `make lint`       — fmt + clippy
- `make test`       — cargo test
- `make build`      — release build
- `make docker`     — build Docker image

Tailwind (if you want manual control):
```bash
npx tailwindcss -i assets/tailwind.css -c tailwind.config.js -o target/site/pkg/app.css --watch
```

## Environment variables
See `.env.example`. Required:
- `JWT_SECRET`, `REFRESH_TOKEN_SECRET` (>=32 chars), `CSRF_SECRET` (>=16 chars)
- `DATABASE_URL`, `REDIS_URL`
- `APP_BASE_URL`, `COOKIE_DOMAIN`
- `RUST_LOG` (e.g. `info,server=debug`)
Optional observability: `OTEL_EXPORTER_OTLP_ENDPOINT` (set only if collector is running to avoid connection warnings).

## Architecture
Workspace crates:
- `crates/app`    — Leptos UI (SSR + hydrate)
- `crates/server` — Axum SSR server, API, middleware
- `crates/shared` — DTO, config, errors, types
- `crates/domain` — auth/domain logic
- `crates/db`     — SQLx repositories, migrations
- `crates/cli`    — admin CLI (migrate, seed, users)

## Database & migrations
Migrations live in `migrations/`. Run locally:
```bash
make migrate
```
CI runs migrations against ephemeral Postgres.

## Docker/CI
- Dev compose: `infra/docker/compose.dev.yml` (Postgres + Redis)
- Dockerfile: `infra/docker/Dockerfile`
- GitHub Actions workflow runs fmt, clippy (`-D warnings`), tests, and migrations.

## Notes
- If you see `OpenTelemetry ... Connection refused`, remove/clear `OTEL_EXPORTER_OTLP_ENDPOINT` unless a collector is running.
- Ensure `COOKIE_DOMAIN` matches the host you open (e.g. use `localhost` not `127.0.0.1` if the cookie domain is `localhost`).
