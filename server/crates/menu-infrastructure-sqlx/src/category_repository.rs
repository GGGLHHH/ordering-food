use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_menu_application::{ApplicationError, CategoryRepository, TransactionContext};
use ordering_food_menu_domain::{Category, CategoryId, MenuStatus, StoreId};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const CATEGORIES_STORE_SLUG_UNIQUE_CONSTRAINT: &str = "categories_store_slug_unique";

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

    fn parse_category_id(category_id: &CategoryId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(category_id.as_str())
            .map_err(|_| ApplicationError::validation("category id must be a valid UUID"))
    }

    fn parse_store_id(store_id: &StoreId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(store_id.as_str())
            .map_err(|_| ApplicationError::validation("store id must be a valid UUID"))
    }

    fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if error.as_database_error().is_some_and(|database_error| {
            database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
                && database_error.constraint() == Some(CATEGORIES_STORE_SLUG_UNIQUE_CONSTRAINT)
        }) {
            ApplicationError::conflict("category slug already exists in store")
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
        let category_id = Self::parse_category_id(category_id)?;
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
            WHERE id = $1
            "#,
        )
        .bind(category_id)
        .fetch_optional(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load menu category aggregate",
                error,
            )
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(Category::rehydrate(
            CategoryId::new(row.get::<Uuid, _>("id").to_string()),
            StoreId::new(row.get::<Uuid, _>("store_id").to_string()),
            row.get::<String, _>("slug"),
            row.get::<String, _>("name"),
            row.get::<Option<String>, _>("description"),
            row.get::<i32, _>("sort_order"),
            MenuStatus::parse(row.get::<String, _>("status"))?,
            row.get("created_at"),
            row.get("updated_at"),
            row.get("deleted_at"),
        )?))
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        category: &Category,
    ) -> Result<(), ApplicationError> {
        let category_id = Self::parse_category_id(category.id())?;
        let store_id = Self::parse_store_id(category.store_id())?;

        sqlx::query(
            r#"
            INSERT INTO menu.categories (
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
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(category_id)
        .bind(store_id)
        .bind(category.slug())
        .bind(category.name())
        .bind(category.description())
        .bind(category.sort_order())
        .bind(category.status().as_str())
        .bind(category.created_at())
        .bind(category.updated_at())
        .bind(category.deleted_at())
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| Self::map_write_error("failed to insert menu category", error))?;

        Ok(())
    }
}
