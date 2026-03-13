use async_trait::async_trait;
use ordering_food_menu_application::{
    ApplicationError, ItemListFilter, ItemReadModel, ItemReadRepository,
};
use ordering_food_menu_domain::{ItemId, StoreId};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxItemReadRepository {
    pool: PgPool,
}

impl SqlxItemReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_store_id(store_id: &StoreId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(store_id.as_str())
            .map_err(|_| ApplicationError::validation("store id must be a valid UUID"))
    }

    fn parse_item_id(item_id: &ItemId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(item_id.as_str())
            .map_err(|_| ApplicationError::validation("item id must be a valid UUID"))
    }
}

#[async_trait]
impl ItemReadRepository for SqlxItemReadRepository {
    async fn list_active_by_store(
        &self,
        store_id: &StoreId,
        filter: ItemListFilter,
    ) -> Result<Vec<ItemReadModel>, ApplicationError> {
        let store_id = Self::parse_store_id(store_id)?;
        let rows = if let Some(category_id) = filter.category_id {
            let category_id = Uuid::parse_str(category_id.as_str())
                .map_err(|_| ApplicationError::validation("category id must be a valid UUID"))?;
            sqlx::query(
                r#"
                SELECT
                    id,
                    store_id,
                    category_id,
                    slug,
                    name,
                    description,
                    image_url,
                    price_amount,
                    sort_order,
                    status,
                    created_at,
                    updated_at,
                    deleted_at
                FROM menu.items
                WHERE store_id = $1
                  AND category_id = $2
                  AND status = 'active'
                  AND deleted_at IS NULL
                ORDER BY sort_order ASC, id ASC
                "#,
            )
            .bind(store_id)
            .bind(category_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT
                    id,
                    store_id,
                    category_id,
                    slug,
                    name,
                    description,
                    image_url,
                    price_amount,
                    sort_order,
                    status,
                    created_at,
                    updated_at,
                    deleted_at
                FROM menu.items
                WHERE store_id = $1
                  AND status = 'active'
                  AND deleted_at IS NULL
                ORDER BY sort_order ASC, id ASC
                "#,
            )
            .bind(store_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query menu items", error)
        })?;

        Ok(rows
            .into_iter()
            .map(|row| ItemReadModel {
                item_id: row.get::<Uuid, _>("id").to_string(),
                store_id: row.get::<Uuid, _>("store_id").to_string(),
                category_id: row.get::<Uuid, _>("category_id").to_string(),
                slug: row.get("slug"),
                name: row.get("name"),
                description: row.get("description"),
                image_url: row.get("image_url"),
                price_amount: row.get("price_amount"),
                sort_order: row.get("sort_order"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
            })
            .collect())
    }

    async fn get_active_by_id(
        &self,
        item_id: &ItemId,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        let item_id = Self::parse_item_id(item_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                store_id,
                category_id,
                slug,
                name,
                description,
                image_url,
                price_amount,
                sort_order,
                status,
                created_at,
                updated_at,
                deleted_at
            FROM menu.items
            WHERE id = $1
              AND status = 'active'
              AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query menu item", error)
        })?;

        Ok(row.map(|row| ItemReadModel {
            item_id: row.get::<Uuid, _>("id").to_string(),
            store_id: row.get::<Uuid, _>("store_id").to_string(),
            category_id: row.get::<Uuid, _>("category_id").to_string(),
            slug: row.get("slug"),
            name: row.get("name"),
            description: row.get("description"),
            image_url: row.get("image_url"),
            price_amount: row.get("price_amount"),
            sort_order: row.get("sort_order"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }
}
