use async_trait::async_trait;
use ordering_food_menu_application::{ApplicationError, StoreReadModel, StoreReadRepository};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxStoreReadRepository {
    pool: PgPool,
}

impl SqlxStoreReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StoreReadRepository for SqlxStoreReadRepository {
    async fn get_active(&self) -> Result<Option<StoreReadModel>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                slug,
                name,
                currency_code,
                timezone,
                status,
                created_at,
                updated_at,
                deleted_at
            FROM menu.stores
            WHERE status = 'active' AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query active menu store", error)
        })?;

        Ok(row.map(|row| StoreReadModel {
            store_id: row.get::<Uuid, _>("id").to_string(),
            slug: row.get("slug"),
            name: row.get("name"),
            currency_code: row.get("currency_code"),
            timezone: row.get("timezone"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }
}
