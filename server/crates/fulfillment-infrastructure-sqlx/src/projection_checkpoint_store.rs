use ordering_food_fulfillment_application::ApplicationError;
use ordering_food_shared_kernel::Timestamp;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlxProjectionCheckpoint {
    pub projector_name: String,
    pub last_processed_id: i64,
    pub updated_at: Timestamp,
}

#[derive(Clone)]
pub struct SqlxProjectionCheckpointStore {
    pool: PgPool,
}

impl SqlxProjectionCheckpointStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SqlxProjectionCheckpointStore {
    pub async fn get(
        &self,
        projector_name: &str,
    ) -> Result<SqlxProjectionCheckpoint, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT
                projector_name,
                last_processed_id,
                updated_at
            FROM platform.projection_checkpoints
            WHERE projector_name = $1
            LIMIT 1
            "#,
        )
        .bind(projector_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query projection checkpoint", error)
        })?;

        Ok(row.map_or(
            SqlxProjectionCheckpoint {
                projector_name: projector_name.to_string(),
                last_processed_id: 0,
                updated_at: Timestamp::UNIX_EPOCH,
            },
            |row| SqlxProjectionCheckpoint {
                projector_name: row.get("projector_name"),
                last_processed_id: row.get("last_processed_id"),
                updated_at: row.get("updated_at"),
            },
        ))
    }

    pub async fn save(
        &self,
        projector_name: &str,
        last_processed_id: i64,
        updated_at: Timestamp,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO platform.projection_checkpoints (
                projector_name,
                last_processed_id,
                updated_at
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (projector_name) DO UPDATE
            SET
                last_processed_id = EXCLUDED.last_processed_id,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(projector_name)
        .bind(last_processed_id)
        .bind(updated_at)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to persist projection checkpoint",
                error,
            )
        })?;

        Ok(())
    }
}
