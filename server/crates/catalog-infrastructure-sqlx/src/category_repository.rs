use crate::{parse_uuid, transaction::SqlxTransactionContext};
use async_trait::async_trait;
use ordering_food_catalog_application::{ApplicationError, CategoryRepository, TransactionContext};
use ordering_food_catalog_domain::{BrandCatalogId, Category, CategoryId};
use sqlx::{Postgres, Row, Transaction};

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const CATEGORIES_BRAND_CATALOG_SLUG_UNIQUE_CONSTRAINT: &str =
    "catalog_categories_brand_catalog_slug_unique";

#[derive(Debug, Default)]
pub struct SqlxCategoryRepository;

impl SqlxCategoryRepository {
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

    fn map_row(row: sqlx::postgres::PgRow) -> Result<Category, ApplicationError> {
        Category::create(
            CategoryId::new(row.get::<uuid::Uuid, _>("id").to_string()),
            BrandCatalogId::new(row.get::<uuid::Uuid, _>("brand_catalog_id").to_string()),
            row.get::<String, _>("slug"),
            row.get::<String, _>("name"),
            row.get::<Option<String>, _>("description"),
            row.get::<i32, _>("sort_order"),
            row.get("created_at"),
        )
        .map_err(Into::into)
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint()
                    == Some(CATEGORIES_BRAND_CATALOG_SLUG_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("category slug already exists in brand catalog")
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }
}

#[async_trait]
impl CategoryRepository for SqlxCategoryRepository {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        category_id: &CategoryId,
    ) -> Result<Option<Category>, ApplicationError> {
        let category_id = parse_uuid(category_id.as_str(), "category id")?;
        let row = sqlx::query(
            r#"
            SELECT id, brand_catalog_id, slug, name, description, sort_order, created_at, updated_at
            FROM catalog.categories
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(category_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to load category aggregate", error)
        })?;

        row.map(Self::map_row).transpose()
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        category: &Category,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO catalog.categories (
                id,
                brand_catalog_id,
                slug,
                name,
                description,
                sort_order,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#,
        )
        .bind(parse_uuid(category.id().as_str(), "category id")?)
        .bind(parse_uuid(
            category.brand_catalog_id().as_str(),
            "brand catalog id",
        )?)
        .bind(category.slug())
        .bind(category.name())
        .bind(category.description())
        .bind(category.sort_order())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert category", error))?;

        Ok(())
    }
}
