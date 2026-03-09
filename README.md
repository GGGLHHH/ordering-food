# ordering-food

Rust backend API scaffold for the `server` directory, with local development dependencies managed by Docker Compose.

## Prerequisites

- Docker and Docker Compose
- Rust stable toolchain managed by `rustup`
- Bacon for hot reload: `cargo install --locked bacon`
- SQLx CLI for manual migration management: `cargo install --locked sqlx-cli --no-default-features --features postgres,rustls`

## Start dependencies

```bash
make up
```

This starts local infrastructure dependencies only:

- PostgreSQL `18.3` on `127.0.0.1:5432`
- Redis `8.6.1` on `127.0.0.1:6379`

## Run the API server locally

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

## Run the full containerized stack

```bash
make compose-up
```

This builds and starts the full Docker Compose stack:

- PostgreSQL
- Redis
- `ordering-food-server`
- `autoheal` for unhealthy container restarts

## Manage database migrations

```bash
make migration-info
make migration-up
make migration-down
make migration-create NAME=add_orders_table
```

These commands invoke `cargo sqlx migrate ...` inside `/server`. `migration-up`, `migration-down`, and `migration-info` read `DATABASE__URL` from the root `.env` file, while `migration-create` creates a reversible migration skeleton by default.

## Build a container image

```bash
docker build -f server/Dockerfile -t ordering-food-server:local server
```

The runtime image defaults to `APP__HOST=0.0.0.0` and `APP__PORT=8080`. If you pass the root `.env` file into the container, override `APP__HOST` back to `0.0.0.0`, otherwise the server will only bind to the container loopback interface.

## Available endpoints

- `GET /health/live`
- `GET /health/ready`
- `GET /openapi.json`
- `GET /docs`

## Configuration

The server automatically loads the root `.env` file on startup and then applies environment variables with double underscores as separators:

- `APP__HOST`
- `APP__PORT`
- `APP__AUTO_MIGRATE`
- `APP__ALLOWED_ORIGINS`
- `DATABASE__URL`
- `DATABASE__MAX_CONNECTIONS`
- `REDIS__URL`

Default local development values live in the root `.env` file.

Example override for one-off runs:

```bash
APP__PORT=9090 DATABASE__MAX_CONNECTIONS=20 make run
```

## Make shortcuts

```bash
make help
```

Common commands:

- `make up`
- `make compose-up`
- `make down`
- `make ps`
- `make logs`
- `make run`
- `make dev`
- `make migration-info`
- `make migration-up`
- `make migration-down`
- `make migration-create NAME=add_orders_table`
- `make fmt`
- `make fmt-check`
- `make clippy`
- `make test`
- `make check`

## Validation

```bash
make check
```
