use crate::{parse_uuid, transaction::SqlxTransactionContext};
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, BrandCatalogRepository, TransactionContext,
};
use ordering_food_catalog_domain::{BrandCatalog, BrandCatalogId, BrandId};
use sqlx::{Postgres, Row, Transaction};

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const BRAND_CATALOGS_BRAND_ID_UNIQUE_CONSTRAINT: &str = "catalog_brand_catalogs_brand_id_unique";
const BRAND_CATALOGS_SLUG_UNIQUE_CONSTRAINT: &str = "catalog_brand_catalogs_slug_unique";

#[derive(Debug, Default)]
pub struct SqlxBrandCatalogRepository;

impl SqlxBrandCatalogRepository {
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

    fn map_row(row: sqlx::postgres::PgRow) -> Result<BrandCatalog, ApplicationError> {
        BrandCatalog::create(
            BrandCatalogId::new(row.get::<uuid::Uuid, _>("id").to_string()),
            BrandId::new(row.get::<uuid::Uuid, _>("brand_id").to_string()),
            row.get::<String, _>("slug"),
            row.get::<String, _>("name"),
            row.get("created_at"),
        )
        .map_err(Into::into)
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(BRAND_CATALOGS_BRAND_ID_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("brand catalog already exists for brand scope")
        } else if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(BRAND_CATALOGS_SLUG_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("brand catalog slug already exists")
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }
}

#[async_trait]
impl BrandCatalogRepository for SqlxBrandCatalogRepository {
    async fn find_by_brand_id(
        &self,
        tx: &mut dyn TransactionContext,
        brand_id: &BrandId,
    ) -> Result<Option<BrandCatalog>, ApplicationError> {
        let brand_id = parse_uuid(brand_id.as_str(), "brand id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, slug, name, created_at, updated_at
            FROM catalog.brand_catalogs
            WHERE brand_id = $1
            LIMIT 1
            "#,
        )
        .bind(brand_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load brand catalog by brand scope",
                error,
            )
        })?;

        row.map(Self::map_row).transpose()
    }

    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        brand_catalog_id: &BrandCatalogId,
    ) -> Result<Option<BrandCatalog>, ApplicationError> {
        let brand_catalog_id = parse_uuid(brand_catalog_id.as_str(), "brand catalog id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_id, slug, name, created_at, updated_at
            FROM catalog.brand_catalogs
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(brand_catalog_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load brand catalog aggregate",
                error,
            )
        })?;

        row.map(Self::map_row).transpose()
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        brand_catalog: &BrandCatalog,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO catalog.brand_catalogs (
                id,
                brand_id,
                slug,
                name,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(parse_uuid(brand_catalog.id().as_str(), "brand catalog id")?)
        .bind(parse_uuid(brand_catalog.brand_id().as_str(), "brand id")?)
        .bind(brand_catalog.slug())
        .bind(brand_catalog.name())
        .bind(brand_catalog.created_at())
        .bind(brand_catalog.updated_at())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert brand catalog", error))?;

        Ok(())
    }
}
