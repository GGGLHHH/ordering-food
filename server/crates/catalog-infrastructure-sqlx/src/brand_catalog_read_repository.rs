use crate::parse_uuid;
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, BrandCatalogReadModel, BrandCatalogReadRepository,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct SqlxBrandCatalogReadRepository {
    pool: PgPool,
}

impl SqlxBrandCatalogReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        brand_catalog_id: &str,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError> {
        let brand_catalog_id = parse_uuid(brand_catalog_id, "brand catalog id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, slug, name, created_at, updated_at
            FROM catalog.brand_catalogs
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(brand_catalog_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query brand catalog read model",
                error,
            )
        })?;

        Ok(row.map(map_brand_catalog_row))
    }

    pub async fn find_by_brand_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError> {
        let brand_id = parse_uuid(brand_id, "brand id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, slug, name, created_at, updated_at
            FROM catalog.brand_catalogs
            WHERE brand_id = $1
            LIMIT 1
            "#,
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query brand catalog by brand scope",
                error,
            )
        })?;

        Ok(row.map(map_brand_catalog_row))
    }
}

fn map_brand_catalog_row(row: sqlx::postgres::PgRow) -> BrandCatalogReadModel {
    BrandCatalogReadModel {
        brand_catalog_id: row.get::<uuid::Uuid, _>("id").to_string(),
        brand_id: row.get::<uuid::Uuid, _>("brand_id").to_string(),
        slug: row.get("slug"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl BrandCatalogReadRepository for SqlxBrandCatalogReadRepository {
    async fn find_by_id(
        &self,
        brand_catalog_id: &str,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError> {
        SqlxBrandCatalogReadRepository::find_by_id(self, brand_catalog_id).await
    }

    async fn find_by_brand_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError> {
        SqlxBrandCatalogReadRepository::find_by_brand_id(self, brand_id).await
    }
}
