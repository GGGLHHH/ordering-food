use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_menu_application::{ApplicationError, StoreRepository, TransactionContext};
use ordering_food_menu_domain::{MenuStatus, Store, StoreId};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const STORES_SLUG_UNIQUE_CONSTRAINT: &str = "stores_slug_unique";

#[derive(Debug, Default)]
pub struct SqlxStoreRepository;

impl SqlxStoreRepository {
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

    fn parse_store_id(store_id: &StoreId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(store_id.as_str())
            .map_err(|_| ApplicationError::validation("store id must be a valid UUID"))
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(STORES_SLUG_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("store slug already exists")
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }
}

#[async_trait]
impl StoreRepository for SqlxStoreRepository {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_id: &StoreId,
    ) -> Result<Option<Store>, ApplicationError> {
        let store_id = Self::parse_store_id(store_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                slug,
                name,
                currency_code,
                timezone,
                status,
                created_at,
                updated_at,
                deleted_at
            FROM menu.stores
            WHERE id = $1
            "#,
        )
        .bind(store_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to load menu store aggregate", error)
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(Store::rehydrate(
            StoreId::new(row.get::<Uuid, _>("id").to_string()),
            row.get::<String, _>("slug"),
            row.get::<String, _>("name"),
            row.get::<String, _>("currency_code"),
            row.get::<String, _>("timezone"),
            MenuStatus::parse(row.get::<String, _>("status"))?,
            row.get("created_at"),
            row.get("updated_at"),
            row.get("deleted_at"),
        )?))
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        store: &Store,
    ) -> Result<(), ApplicationError> {
        let store_id = Self::parse_store_id(store.id())?;
        sqlx::query(
            r#"
            INSERT INTO menu.stores (
                id,
                slug,
                name,
                currency_code,
                timezone,
                status,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(store_id)
        .bind(store.slug())
        .bind(store.name())
        .bind(store.currency_code())
        .bind(store.timezone())
        .bind(store.status().as_str())
        .bind(store.created_at())
        .bind(store.updated_at())
        .bind(store.deleted_at())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert menu store", error))?;

        Ok(())
    }
}
