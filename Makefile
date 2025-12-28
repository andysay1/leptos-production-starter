COMPOSE ?= docker compose
DATABASE_URL ?= postgres://postgres:postgres@localhost:5432/leptos_app
REDIS_URL ?= redis://localhost:6379
RUST_LOG ?= info,server=debug
LEPTOS ?= cargo leptos

.PHONY: dev lint test fmt clippy migrate migrate-check seed build docker clean

dev:
	$(COMPOSE) -f infra/docker/compose.dev.yml up -d db redis
	APP_ENV=development DATABASE_URL=$(DATABASE_URL) REDIS_URL=$(REDIS_URL) RUST_LOG=$(RUST_LOG) \
		$(LEPTOS) watch --split -- --bin server

lint: fmt clippy

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

migrate:
	DATABASE_URL=$(DATABASE_URL) cargo sqlx migrate run

migrate-check:
	DATABASE_URL=$(DATABASE_URL) cargo sqlx migrate check

seed:
	DATABASE_URL=$(DATABASE_URL) REDIS_URL=$(REDIS_URL) cargo run -p cli -- seed

build:
	$(LEPTOS) build --release --precompress --split -- --bin server

docker:
	$(COMPOSE) -f infra/docker/compose.dev.yml build

clean:
	cargo clean
	$(COMPOSE) -f infra/docker/compose.dev.yml down -v
