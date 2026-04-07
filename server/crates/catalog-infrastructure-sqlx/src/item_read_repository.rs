use crate::parse_uuid;
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, CatalogItemListFilter, ItemReadModel, ItemReadRepository,
    StoreItemListingReadModel, StoreItemListingReadRepository,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct SqlxItemReadRepository {
    pool: PgPool,
}

impl SqlxItemReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &str,
        category_id: Option<&str>,
    ) -> Result<Vec<ItemReadModel>, ApplicationError> {
        let brand_catalog_id = parse_uuid(brand_catalog_id, "brand catalog id")?;
        let rows = if let Some(category_id) = category_id {
            let category_id = parse_uuid(category_id, "category id")?;
            sqlx::query(
                r#"
                SELECT id, brand_catalog_id, category_id, slug, name, description, image_url, sort_order, created_at, updated_at
                FROM catalog.items
                WHERE brand_catalog_id = $1
                  AND category_id = $2
                ORDER BY sort_order ASC, id ASC
                "#,
            )
            .bind(brand_catalog_id)
            .bind(category_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, brand_catalog_id, category_id, slug, name, description, image_url, sort_order, created_at, updated_at
                FROM catalog.items
                WHERE brand_catalog_id = $1
                ORDER BY sort_order ASC, id ASC
                "#,
            )
            .bind(brand_catalog_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query catalog items", error)
        })?;

        Ok(rows.into_iter().map(map_item_row).collect())
    }

    pub async fn find_by_id(
        &self,
        item_id: &str,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        let item_id = parse_uuid(item_id, "item id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_catalog_id, category_id, slug, name, description, image_url, sort_order, created_at, updated_at
            FROM catalog.items
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query catalog item", error)
        })?;

        Ok(row.map(map_item_row))
    }

    pub async fn find_listing(
        &self,
        store_catalog_id: &str,
        item_id: &str,
    ) -> Result<Option<StoreItemListingReadModel>, ApplicationError> {
        let store_catalog_id = parse_uuid(store_catalog_id, "store catalog id")?;
        let item_id = parse_uuid(item_id, "item id")?;
        let row = sqlx::query(
            r#"
            SELECT store_catalog_id, item_id, price_amount, status, display_rule, created_at, updated_at
            FROM catalog.store_item_listings
            WHERE store_catalog_id = $1
              AND item_id = $2
            LIMIT 1
            "#,
        )
        .bind(store_catalog_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query store item listing",
                error,
            )
        })?;

        Ok(row.map(map_listing_row))
    }

    pub async fn list_listings_by_store_catalog_id(
        &self,
        store_catalog_id: &str,
    ) -> Result<Vec<StoreItemListingReadModel>, ApplicationError> {
        let store_catalog_id = parse_uuid(store_catalog_id, "store catalog id")?;
        let rows = sqlx::query(
            r#"
            SELECT store_catalog_id, item_id, price_amount, status, display_rule, created_at, updated_at
            FROM catalog.store_item_listings
            WHERE store_catalog_id = $1
            ORDER BY item_id ASC
            "#,
        )
        .bind(store_catalog_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query store item listings",
                error,
            )
        })?;

        Ok(rows.into_iter().map(map_listing_row).collect())
    }
}

fn map_item_row(row: sqlx::postgres::PgRow) -> ItemReadModel {
    ItemReadModel {
        item_id: row.get::<uuid::Uuid, _>("id").to_string(),
        brand_catalog_id: row.get::<uuid::Uuid, _>("brand_catalog_id").to_string(),
        category_id: row.get::<uuid::Uuid, _>("category_id").to_string(),
        slug: row.get("slug"),
        name: row.get("name"),
        description: row.get("description"),
        image_url: row.get("image_url"),
        sort_order: row.get("sort_order"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn map_listing_row(row: sqlx::postgres::PgRow) -> StoreItemListingReadModel {
    StoreItemListingReadModel {
        store_catalog_id: row.get::<uuid::Uuid, _>("store_catalog_id").to_string(),
        item_id: row.get::<uuid::Uuid, _>("item_id").to_string(),
        price_amount: row.get("price_amount"),
        status: row.get("status"),
        display_rule: row.get("display_rule"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl ItemReadRepository for SqlxItemReadRepository {
    async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &str,
        filter: CatalogItemListFilter,
    ) -> Result<Vec<ItemReadModel>, ApplicationError> {
        SqlxItemReadRepository::list_by_brand_catalog_id(
            self,
            brand_catalog_id,
            filter
                .category_id
                .as_ref()
                .map(|category_id| category_id.as_str()),
        )
        .await
    }

    async fn find_by_id(
        &self,
        item_id: &str,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        SqlxItemReadRepository::find_by_id(self, item_id).await
    }

    async fn find_by_slug(
        &self,
        brand_catalog_id: &str,
        slug: &str,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        let mut items =
            SqlxItemReadRepository::list_by_brand_catalog_id(self, brand_catalog_id, None)
                .await?;
        Ok(items
            .drain(..)
            .find(|item| item.slug == slug.trim().to_ascii_lowercase()))
    }
}

#[async_trait]
impl StoreItemListingReadRepository for SqlxItemReadRepository {
    async fn find_by_item_id(
        &self,
        store_catalog_id: &str,
        item_id: &str,
    ) -> Result<Option<StoreItemListingReadModel>, ApplicationError> {
        SqlxItemReadRepository::find_listing(self, store_catalog_id, item_id)
            .await
    }

    async fn list_by_store_catalog_id(
        &self,
        store_catalog_id: &str,
    ) -> Result<Vec<StoreItemListingReadModel>, ApplicationError> {
        SqlxItemReadRepository::list_listings_by_store_catalog_id(self, store_catalog_id)
            .await
    }
}
