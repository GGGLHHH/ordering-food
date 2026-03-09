FROM rust:1.94.0-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --locked --bin ordering-food-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --home-dir /app --shell /usr/sbin/nologin appuser

WORKDIR /app

COPY --from=builder /app/target/release/ordering-food-server /usr/local/bin/ordering-food-server

ENV APP__HOST=0.0.0.0 \
    APP__PORT=8080

EXPOSE 8080

USER appuser

CMD ["ordering-food-server"]
