CREATE SCHEMA IF NOT EXISTS platform;

CREATE TABLE platform.outbox_messages (
    id BIGSERIAL PRIMARY KEY,
    producer_context TEXT NOT NULL,
    event_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    available_at TIMESTAMPTZ NOT NULL,
    error_count INTEGER NOT NULL DEFAULT 0 CHECK (error_count >= 0),
    last_error TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_platform_outbox_messages_available_at_id
    ON platform.outbox_messages (available_at, id);

CREATE TABLE platform.projection_checkpoints (
    projector_name TEXT PRIMARY KEY,
    last_processed_id BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL
);
