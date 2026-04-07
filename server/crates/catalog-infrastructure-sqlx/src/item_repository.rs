use crate::{parse_uuid, transaction::SqlxTransactionContext};
use async_trait::async_trait;
use ordering_food_catalog_application::{ApplicationError, ItemRepository, TransactionContext};
use ordering_food_catalog_domain::{BrandCatalogId, CategoryId, Item, ItemId};
use sqlx::{Postgres, Row, Transaction};

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const ITEMS_BRAND_CATALOG_SLUG_UNIQUE_CONSTRAINT: &str = "catalog_items_brand_catalog_slug_unique";

#[derive(Debug, Default)]
pub struct SqlxItemRepository;

impl SqlxItemRepository {
    fn transaction(
        tx: &mut dyn TransactionContext,
    ) -> Result<&mut Transaction<'static, Postgres>, ApplicationError> {
        tx.as_any_mut()
            .downcast_mut::<SqlxTransactionContext>()
            .map(SqlxTransactionContext::transaction_mut)
            .ok_or_else(|| {
                ApplicationError::unexpected("unexpected transaction context implementation")
            })
    }

    fn map_row(row: sqlx::postgres::PgRow) -> Result<Item, ApplicationError> {
        Item::create(
            ItemId::new(row.get::<uuid::Uuid, _>("id").to_string()),
            BrandCatalogId::new(row.get::<uuid::Uuid, _>("brand_catalog_id").to_string()),
            CategoryId::new(row.get::<uuid::Uuid, _>("category_id").to_string()),
            row.get::<String, _>("slug"),
            row.get::<String, _>("name"),
            row.get::<Option<String>, _>("description"),
            row.get::<Option<String>, _>("image_url"),
            row.get::<i32, _>("sort_order"),
            row.get("created_at"),
        )
        .map_err(Into::into)
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(ITEMS_BRAND_CATALOG_SLUG_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("item slug already exists in brand catalog")
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }
}

#[async_trait]
impl ItemRepository for SqlxItemRepository {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        item_id: &ItemId,
    ) -> Result<Option<Item>, ApplicationError> {
        let item_id = parse_uuid(item_id.as_str(), "item id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_catalog_id, category_id, slug, name, description, image_url, sort_order, created_at, updated_at
            FROM catalog.items
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to load catalog item aggregate", error)
        })?;

        row.map(Self::map_row).transpose()
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        item: &Item,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO catalog.items (
                id,
                brand_catalog_id,
                category_id,
                slug,
                name,
                description,
                image_url,
                sort_order,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#,
        )
        .bind(parse_uuid(item.id().as_str(), "item id")?)
        .bind(parse_uuid(
            item.brand_catalog_id().as_str(),
            "brand catalog id",
        )?)
        .bind(parse_uuid(item.category_id().as_str(), "category id")?)
        .bind(item.slug())
        .bind(item.name())
        .bind(item.description())
        .bind(item.image_url())
        .bind(item.sort_order())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert catalog item", error))?;

        Ok(())
    }
}
