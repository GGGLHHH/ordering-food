use crate::parse_uuid;
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, StoreCatalogReadModel, StoreCatalogReadRepository,
};
use ordering_food_catalog_domain::{StoreCatalogId, StoreId};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct SqlxStoreCatalogReadRepository {
    pool: PgPool,
}

impl SqlxStoreCatalogReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        store_catalog_id: &str,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError> {
        let store_catalog_id = parse_uuid(store_catalog_id, "store catalog id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, store_id, status, display_rule, created_at, updated_at
            FROM catalog.store_catalogs
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(store_catalog_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query store catalog read model",
                error,
            )
        })?;

        Ok(row.map(map_store_catalog_row))
    }

    pub async fn find_by_store_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError> {
        let store_id = parse_uuid(store_id, "store id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, store_id, status, display_rule, created_at, updated_at
            FROM catalog.store_catalogs
            WHERE store_id = $1
            LIMIT 1
            "#,
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query store catalog by store scope",
                error,
            )
        })?;

        Ok(row.map(map_store_catalog_row))
    }
}

fn map_store_catalog_row(row: sqlx::postgres::PgRow) -> StoreCatalogReadModel {
    StoreCatalogReadModel {
        store_catalog_id: row.get::<uuid::Uuid, _>("id").to_string(),
        brand_id: row.get::<uuid::Uuid, _>("brand_id").to_string(),
        store_id: row.get::<uuid::Uuid, _>("store_id").to_string(),
        status: row.get("status"),
        display_rule: row.get("display_rule"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl StoreCatalogReadRepository for SqlxStoreCatalogReadRepository {
    async fn find_by_id(
        &self,
        store_catalog_id: &StoreCatalogId,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError> {
        SqlxStoreCatalogReadRepository::find_by_id(self, store_catalog_id.as_str()).await
    }

    async fn find_by_store_id(
        &self,
        store_id: &StoreId,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError> {
        SqlxStoreCatalogReadRepository::find_by_store_id(self, store_id.as_str()).await
    }
}
