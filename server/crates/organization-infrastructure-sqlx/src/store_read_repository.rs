use async_trait::async_trait;
use ordering_food_organization_application::{
    ApplicationError, StoreReadModel, StoreReadRepository,
};
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

    fn parse_store_id(store_id: &str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(store_id)
            .map_err(|_| ApplicationError::validation("store id must be a valid UUID"))
    }
}

#[async_trait]
impl StoreReadRepository for SqlxStoreReadRepository {
    async fn get_active(&self) -> Result<Option<StoreReadModel>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                brand_id,
                slug,
                name,
                currency_code,
                timezone,
                status
            FROM organization.stores
            WHERE status = 'active' AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query active organization store",
                error,
            )
        })?;

        Ok(row.map(|row| StoreReadModel {
            store_id: row.get::<Uuid, _>("id").to_string(),
            brand_id: row.get::<Uuid, _>("brand_id").to_string(),
            slug: row.get("slug"),
            name: row.get("name"),
            currency_code: row.get("currency_code"),
            timezone: row.get("timezone"),
            status: row.get("status"),
        }))
    }

    async fn get_by_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreReadModel>, ApplicationError> {
        let store_id = Self::parse_store_id(store_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                brand_id,
                slug,
                name,
                currency_code,
                timezone,
                status
            FROM organization.stores
            WHERE id = $1 AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query organization store", error)
        })?;

        Ok(row.map(|row| StoreReadModel {
            store_id: row.get::<Uuid, _>("id").to_string(),
            brand_id: row.get::<Uuid, _>("brand_id").to_string(),
            slug: row.get("slug"),
            name: row.get("name"),
            currency_code: row.get("currency_code"),
            timezone: row.get("timezone"),
            status: row.get("status"),
        }))
    }
}
