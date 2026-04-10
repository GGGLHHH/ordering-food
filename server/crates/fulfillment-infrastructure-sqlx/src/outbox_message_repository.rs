use ordering_food_fulfillment_application::ApplicationError;
use ordering_food_shared_kernel::Timestamp;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, PartialEq)]
pub struct SqlxOutboxMessageRecord {
    pub id: i64,
    pub producer_context: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub occurred_at: Timestamp,
    pub available_at: Timestamp,
    pub error_count: i32,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
}

#[derive(Clone)]
pub struct SqlxOutboxMessageRepository {
    pool: PgPool,
}

impl SqlxOutboxMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SqlxOutboxMessageRepository {
    pub async fn list_available(
        &self,
        producer_context: &str,
        after_id: i64,
        available_before: Timestamp,
        limit: i64,
    ) -> Result<Vec<SqlxOutboxMessageRecord>, ApplicationError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                producer_context,
                event_type,
                aggregate_id,
                payload,
                occurred_at,
                available_at,
                error_count,
                last_error,
                created_at
            FROM platform.outbox_messages
            WHERE producer_context = $1
              AND id > $2
              AND available_at <= $3
              AND error_count = 0
            ORDER BY id ASC
            LIMIT $4
            "#,
        )
        .bind(producer_context)
        .bind(after_id)
        .bind(available_before)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query outbox messages", error)
        })?;

        Ok(rows
            .into_iter()
            .map(|row| SqlxOutboxMessageRecord {
                id: row.get("id"),
                producer_context: row.get("producer_context"),
                event_type: row.get("event_type"),
                aggregate_id: row.get("aggregate_id"),
                payload: row.get("payload"),
                occurred_at: row.get("occurred_at"),
                available_at: row.get("available_at"),
                error_count: row.get("error_count"),
                last_error: row.get("last_error"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    pub async fn record_failure(
        &self,
        message_id: i64,
        last_error: &str,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            UPDATE platform.outbox_messages
            SET
                error_count = error_count + 1,
                last_error = $2
            WHERE id = $1
            "#,
        )
        .bind(message_id)
        .bind(last_error)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to record projector failure", error)
        })?;

        Ok(())
    }
}
