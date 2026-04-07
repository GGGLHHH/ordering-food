use crate::{
    display_rule_as_str, parse_display_rule, parse_sellable_status, parse_uuid,
    sellable_status_as_str, transaction::SqlxTransactionContext,
};
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, StoreCatalogRepository, TransactionContext,
};
use ordering_food_catalog_domain::{BrandId, StoreCatalog, StoreCatalogId, StoreId};
use sqlx::{Postgres, Row, Transaction};

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const STORE_CATALOGS_STORE_ID_UNIQUE_CONSTRAINT: &str = "catalog_store_catalogs_store_id_unique";

#[derive(Debug, Default)]
pub struct SqlxStoreCatalogRepository;

impl SqlxStoreCatalogRepository {
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

    fn map_row(row: sqlx::postgres::PgRow) -> Result<StoreCatalog, ApplicationError> {
        StoreCatalog::attach(
            StoreCatalogId::new(row.get::<uuid::Uuid, _>("id").to_string()),
            BrandId::new(row.get::<uuid::Uuid, _>("brand_id").to_string()),
            StoreId::new(row.get::<uuid::Uuid, _>("store_id").to_string()),
            parse_sellable_status(&row.get::<String, _>("status"))?,
            parse_display_rule(&row.get::<String, _>("display_rule"))?,
            row.get("created_at"),
        )
        .map_err(Into::into)
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(STORE_CATALOGS_STORE_ID_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("store catalog already exists for store scope")
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }
}

#[async_trait]
impl StoreCatalogRepository for SqlxStoreCatalogRepository {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_catalog_id: &StoreCatalogId,
    ) -> Result<Option<StoreCatalog>, ApplicationError> {
        let store_catalog_id = parse_uuid(store_catalog_id.as_str(), "store catalog id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, store_id, status, display_rule, created_at, updated_at
            FROM catalog.store_catalogs
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(store_catalog_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load store catalog aggregate",
                error,
            )
        })?;

        row.map(Self::map_row).transpose()
    }

    async fn find_by_store_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_id: &StoreId,
    ) -> Result<Option<StoreCatalog>, ApplicationError> {
        let store_id = parse_uuid(store_id.as_str(), "store id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, store_id, status, display_rule, created_at, updated_at
            FROM catalog.store_catalogs
            WHERE store_id = $1
            LIMIT 1
            "#,
        )
        .bind(store_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load store catalog by store scope",
                error,
            )
        })?;

        row.map(Self::map_row).transpose()
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        store_catalog: &StoreCatalog,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO catalog.store_catalogs (
                id,
                brand_id,
                store_id,
                status,
                display_rule,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(parse_uuid(store_catalog.id().as_str(), "store catalog id")?)
        .bind(parse_uuid(store_catalog.brand_id().as_str(), "brand id")?)
        .bind(parse_uuid(store_catalog.store_id().as_str(), "store id")?)
        .bind(sellable_status_as_str(store_catalog.status()))
        .bind(display_rule_as_str(store_catalog.display_rule()))
        .bind(store_catalog.created_at())
        .bind(store_catalog.updated_at())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert store catalog", error))?;

        Ok(())
    }
}
