# ordering-food

Rust backend API scaffold for the `server` directory, with local development dependencies managed by Docker Compose.

## Prerequisites

- Docker and Docker Compose
- Rust stable toolchain managed by `rustup`

## Start dependencies

```bash
docker compose up -d
```

This starts:

- PostgreSQL `18.3` on `127.0.0.1:5432`
- Redis `8.6.1` on `127.0.0.1:6379`

## Run the API server

```bash
cd server
cargo run
```

The server listens on `0.0.0.0:8080` by default.

## Available endpoints

- `GET /health/live`
- `GET /health/ready`
- `GET /openapi.json`
- `GET /docs`

## Configuration

The server uses environment variables with double underscores as separators:

- `APP__HOST`
- `APP__PORT`
- `APP__AUTO_MIGRATE`
- `APP__ALLOWED_ORIGINS`
- `DATABASE__URL`
- `DATABASE__MAX_CONNECTIONS`
- `REDIS__URL`

Example:

```bash
cd server
APP__PORT=9090 DATABASE__MAX_CONNECTIONS=20 cargo run
```

## Validation

```bash
cd server
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
