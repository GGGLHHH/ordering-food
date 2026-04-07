use async_trait::async_trait;
use ordering_food_organization_application::{ApplicationError, BrandReadRepository, BrandRef};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxBrandReadRepository {
    pool: PgPool,
}

impl SqlxBrandReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_brand_id(brand_id: &str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(brand_id)
            .map_err(|_| ApplicationError::validation("brand id must be a valid UUID"))
    }
}

#[async_trait]
impl BrandReadRepository for SqlxBrandReadRepository {
    async fn get_by_id(&self, brand_id: &str) -> Result<Option<BrandRef>, ApplicationError> {
        let brand_id = Self::parse_brand_id(brand_id)?;
        let row = sqlx::query(
            r#"
            SELECT id
            FROM organization.brands
            WHERE id = $1 AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query organization brand", error)
        })?;

        Ok(row.map(|row| BrandRef {
            brand_id: row.get::<Uuid, _>("id").to_string(),
        }))
    }
}
