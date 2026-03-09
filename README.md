# ordering-food

Rust backend API scaffold for the `server` directory, with local development dependencies managed by Docker Compose.

## Prerequisites

- Docker and Docker Compose
- Rust stable toolchain managed by `rustup`
- Bacon for hot reload: `cargo install --locked bacon`

## Start dependencies

```bash
make up
```

This starts:

- PostgreSQL `18.3` on `127.0.0.1:5432`
- Redis `8.6.1` on `127.0.0.1:6379`

## Run the API server

```bash
make run
```

`make run` uses Bacon, so source changes automatically rebuild and restart the server.

If you want one command that starts dependencies first and then enters the hot-reload loop, use:

```bash
make dev
```

This project ships with `/server/bacon.toml`, where the default Bacon job is configured to:

- run `cargo run`
- watch Rust sources plus migrations and Cargo metadata
- kill and restart the server automatically on change

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
APP__PORT=9090 DATABASE__MAX_CONNECTIONS=20 make run
```

## Make shortcuts

```bash
make help
```

Common commands:

- `make up`
- `make down`
- `make ps`
- `make logs`
- `make run`
- `make dev`
- `make fmt`
- `make fmt-check`
- `make clippy`
- `make test`
- `make check`

## Validation

```bash
make check
```
