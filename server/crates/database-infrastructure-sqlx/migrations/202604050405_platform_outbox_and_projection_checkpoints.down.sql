DROP TABLE IF EXISTS platform.projection_checkpoints;
DROP INDEX IF EXISTS platform.idx_platform_outbox_messages_available_at_id;
DROP TABLE IF EXISTS platform.outbox_messages;
DROP SCHEMA IF EXISTS platform;
