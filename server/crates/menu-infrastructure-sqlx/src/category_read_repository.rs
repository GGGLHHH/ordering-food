use async_trait::async_trait;
use ordering_food_menu_application::{ApplicationError, CategoryReadModel, CategoryReadRepository};
use ordering_food_menu_domain::StoreId;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxCategoryReadRepository {
    pool: PgPool,
}

impl SqlxCategoryReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_store_id(store_id: &StoreId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(store_id.as_str())
            .map_err(|_| ApplicationError::validation("store id must be a valid UUID"))
    }
}

#[async_trait]
impl CategoryReadRepository for SqlxCategoryReadRepository {
    async fn list_active_by_store(
        &self,
        store_id: &StoreId,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError> {
        let store_id = Self::parse_store_id(store_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                store_id,
                slug,
                name,
                description,
                sort_order,
                status,
                created_at,
                updated_at,
                deleted_at
            FROM menu.categories
            WHERE store_id = $1
              AND status = 'active'
              AND deleted_at IS NULL
            ORDER BY sort_order ASC, id ASC
            "#,
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query menu categories", error)
        })?;

        Ok(rows
            .into_iter()
            .map(|row| CategoryReadModel {
                category_id: row.get::<Uuid, _>("id").to_string(),
                store_id: row.get::<Uuid, _>("store_id").to_string(),
                slug: row.get("slug"),
                name: row.get("name"),
                description: row.get("description"),
                sort_order: row.get("sort_order"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
            })
            .collect())
    }

    async fn get_active_by_slug(
        &self,
        store_id: &StoreId,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError> {
        let store_id = Self::parse_store_id(store_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                store_id,
                slug,
                name,
                description,
                sort_order,
                status,
                created_at,
                updated_at,
                deleted_at
            FROM menu.categories
            WHERE store_id = $1
              AND slug = $2
              AND status = 'active'
              AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(store_id)
        .bind(slug.trim().to_ascii_lowercase())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query menu category by slug", error)
        })?;

        Ok(row.map(|row| CategoryReadModel {
            category_id: row.get::<Uuid, _>("id").to_string(),
            store_id: row.get::<Uuid, _>("store_id").to_string(),
            slug: row.get("slug"),
            name: row.get("name"),
            description: row.get("description"),
            sort_order: row.get("sort_order"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }
}
