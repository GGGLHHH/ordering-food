use crate::parse_uuid;
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, CategoryReadModel, CategoryReadRepository,
};
use ordering_food_catalog_domain::BrandCatalogId;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct SqlxCategoryReadRepository {
    pool: PgPool,
}

impl SqlxCategoryReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &str,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError> {
        let brand_catalog_id = parse_uuid(brand_catalog_id, "brand catalog id")?;
        let rows = sqlx::query(
            r#"
            SELECT id, brand_catalog_id, slug, name, description, sort_order, created_at, updated_at
            FROM catalog.categories
            WHERE brand_catalog_id = $1
            ORDER BY sort_order ASC, id ASC
            "#,
        )
        .bind(brand_catalog_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query catalog categories", error)
        })?;

        Ok(rows.into_iter().map(map_category_row).collect())
    }

    pub async fn find_by_slug(
        &self,
        brand_catalog_id: &str,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError> {
        let brand_catalog_id = parse_uuid(brand_catalog_id, "brand catalog id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_catalog_id, slug, name, description, sort_order, created_at, updated_at
            FROM catalog.categories
            WHERE brand_catalog_id = $1
              AND slug = $2
            LIMIT 1
            "#,
        )
        .bind(brand_catalog_id)
        .bind(slug.trim().to_ascii_lowercase())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query catalog category by slug",
                error,
            )
        })?;

        Ok(row.map(map_category_row))
    }
}

fn map_category_row(row: sqlx::postgres::PgRow) -> CategoryReadModel {
    CategoryReadModel {
        category_id: row.get::<uuid::Uuid, _>("id").to_string(),
        brand_catalog_id: row.get::<uuid::Uuid, _>("brand_catalog_id").to_string(),
        slug: row.get("slug"),
        name: row.get("name"),
        description: row.get("description"),
        sort_order: row.get("sort_order"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl CategoryReadRepository for SqlxCategoryReadRepository {
    async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &BrandCatalogId,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError> {
        SqlxCategoryReadRepository::list_by_brand_catalog_id(self, brand_catalog_id.as_str()).await
    }

    async fn find_by_slug(
        &self,
        brand_catalog_id: &BrandCatalogId,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError> {
        SqlxCategoryReadRepository::find_by_slug(self, brand_catalog_id.as_str(), slug).await
    }
}
