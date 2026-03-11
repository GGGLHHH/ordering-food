# ordering-food

Rust backend API scaffold for the `server` workspace, now organized as a strict DDD modular monolith.

## Workspace layout

`server` is the Cargo workspace root and currently contains the first bounded context blueprint:

- `apps/api`: the only HTTP entrypoint, responsible for Axum startup, routing, OpenAPI, config, and app-specific composition
- `crates/bootstrap-core`: shared runtime registry kernel for context descriptors, topology planning, migrations, and bootstrap ordering
- `crates/shared-kernel`: minimal cross-context primitives only
- `crates/identity-domain`: pure user-domain model and invariants
- `crates/identity-application`: user use cases and ports
- `crates/identity-infrastructure-sqlx`: SQLx persistence, query read model, and migrations

Within `apps/api`, only `src/composition/**` may depend directly on infrastructure crates. Route handlers and HTTP adapters must stay decoupled from SQLx implementations.

The API app now uses a multi-context composition pipeline:

- `src/composition/platform.rs`: runtime platform dependencies shared by all contexts in the app
- `src/composition/context_registration.rs`: app-specific context registration contract
- `src/composition/registry.rs`: migration/bootstrap orchestration plus lifecycle assembly
- `src/composition/contexts/*.rs`: one adapter per bounded context

The `identity` context uses a dedicated PostgreSQL schema:

- `identity.users`
- `identity.user_profiles`
- `identity.user_identities`

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
- DBHub MCP server on `http://127.0.0.1:1000/mcp`

## Project MCP configuration

This repository checks in project-level MCP configuration for common AI coding clients:

- `.codex/config.toml` for Codex CLI and Codex IDE extension
- `.mcp.json` for Claude Code

The Codex and Claude project configurations both define these MCP servers:

- `dbhub` at `http://localhost:1000/mcp`
- `gitnexus` via `npx -y gitnexus@latest mcp`

Codex loads `.codex/config.toml` only for trusted projects. If Codex reports that project config is disabled, trust the repository once in the TUI or add a user-level trust entry to `~/.codex/config.toml`:

```toml
[projects."/absolute/path/to/ordering-food"]
trust_level = "trusted"
```

After the project is trusted, running `codex` or `codex exec` from this repository will automatically load the project-level MCP servers. You can verify it with:

```bash
codex mcp list
```

This repository also mirrors the Claude `gitnexus` skills into project-level Codex skills under `.codex/skills/gitnexus`, so trusted Codex sessions can invoke the same skill set.

## Run the API server locally


```bash
make run
```

`make run` uses Bacon and runs the workspace binary package:

```bash
cargo run -p ordering-food-api --bin ordering-food-server
```

If you want one command that starts dependencies first and then enters the hot-reload loop, use:

```bash
make dev
```

The Bacon configuration lives in `server/bacon.toml` and watches both `apps/` and `crates/`.

## Run the full containerized stack

```bash
make compose-up
```

This builds and starts the full Docker Compose stack:

- PostgreSQL
- Redis
- `dbhub`
- `ordering-food-server`
- `ordering-food-frontend`
- `ordering-food-nginx`
- `autoheal` for unhealthy container restarts

## Manage database migrations

```bash
make migration-info
make migration-up
make migration-down
make migration-create NAME=add_identity_projection
```

These commands invoke `cargo sqlx migrate ...` inside `/server`, with the source directory fixed to `crates/identity-infrastructure-sqlx/migrations`.

## Build a container image

```bash
docker build -f server/Dockerfile -t ordering-food-server:local server
```

The server runtime image defaults to `APP__HOST=0.0.0.0` and `APP__PORT=8080`. If you pass the root `.env` file into the container, override `APP__HOST` back to `0.0.0.0`, otherwise the server will only bind to the container loopback interface.

The frontend container builds the TanStack Start app and runs the Nitro node server on port `3000`.

The internal Nginx container listens on `127.0.0.1:18081` and is intended to sit behind an external host-level Nginx that terminates HTTPS. It proxies only `/api/*` to the Rust API, proxies `/health` to the frontend Nitro server, and sends all other requests to the frontend app. The Rust API and frontend Nitro services are no longer published directly to the host; external traffic should enter through Nginx only. Backend readiness/docs endpoints remain internal to the Compose network.

## Available endpoints

- `GET /health/live`
- `GET /health/ready`
- `GET /openapi.json`
- `GET /docs`
- `POST /api/examples/echo`
- `GET /api/examples/search?page=1`
- `GET /api/examples/items/{item_id}`
- `POST /api/identity/users`
- `GET /api/identity/users/{user_id}`
- `PATCH /api/identity/users/{user_id}/profile`
- `POST /api/identity/users/{user_id}/identities`
- `POST /api/identity/users/{user_id}/disable`
- `POST /api/identity/users/{user_id}/soft-delete`

The first phase wires the `identity` context end-to-end internally without exposing public business endpoints yet.

## Export frontend TypeScript bindings

The repository uses `ts-rs` from the API contract layer as the single source of truth for frontend bindings.

- Only public HTTP contract types are exported to TypeScript
- Domain, application, infrastructure, runtime, and config types stay backend-internal
- Future business endpoints should define frontend-facing DTOs in `apps/api` and map them to application/domain models explicitly
- The `identity` endpoints already follow this pattern and export their request/response contracts via `ts-rs`

Set `GENERATED_API_DIR` before exporting bindings. The checked-in root `.env` uses the default local path `../frontend/src/api/generated`, and `make export-ts` forwards that environment variable into the export binary.

Generate bindings with:

```bash
make export-ts
```

If you run the binary directly, pass the environment variable explicitly:

```bash
cd server && GENERATED_API_DIR=../frontend/src/api/generated cargo run -p ordering-food-api --bin export-ts-bindings
```

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
- `make export-ts`
- `make migration-info`
- `make migration-up`
- `make migration-down`
- `make migration-create NAME=add_identity_projection`
- `make fmt`
- `make fmt-check`
- `make clippy`
- `make test`
- `make check`

## Validation

```bash
make check
```
