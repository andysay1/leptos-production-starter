FROM rust:1.74 as builder
WORKDIR /app

RUN apt-get update && apt-get install -y libssl-dev pkg-config nodejs npm && rm -rf /var/lib/apt/lists/*

COPY . .
RUN npm install
RUN npm run build
RUN rustup target add wasm32-unknown-unknown
RUN cargo build -p server --release

FROM debian:bullseye-slim
RUN useradd -m app
WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/assets ./assets
COPY --from=builder /app/migrations ./migrations
ENV APP_ENV=production
ENV APP_PORT=3000
EXPOSE 3000
USER app
CMD ["/usr/local/bin/server"]
