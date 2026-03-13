use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_menu_application::{ApplicationError, ItemRepository, TransactionContext};
use ordering_food_menu_domain::Item;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const ITEMS_STORE_SLUG_UNIQUE_CONSTRAINT: &str = "items_store_slug_unique";

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

    fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(value)
            .map_err(|_| ApplicationError::validation(format!("{field} must be a valid UUID")))
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(ITEMS_STORE_SLUG_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("item slug already exists in store")
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }
}

#[async_trait]
impl ItemRepository for SqlxItemRepository {
    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        item: &Item,
    ) -> Result<(), ApplicationError> {
        let item_id = Self::parse_uuid(item.id().as_str(), "item id")?;
        let store_id = Self::parse_uuid(item.store_id().as_str(), "store id")?;
        let category_id = Self::parse_uuid(item.category_id().as_str(), "category id")?;

        sqlx::query(
            r#"
            INSERT INTO menu.items (
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
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(item_id)
        .bind(store_id)
        .bind(category_id)
        .bind(item.slug())
        .bind(item.name())
        .bind(item.description())
        .bind(item.image_url())
        .bind(item.price_amount())
        .bind(item.sort_order())
        .bind(item.status().as_str())
        .bind(item.created_at())
        .bind(item.updated_at())
        .bind(item.deleted_at())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert menu item", error))?;

        Ok(())
    }
}
