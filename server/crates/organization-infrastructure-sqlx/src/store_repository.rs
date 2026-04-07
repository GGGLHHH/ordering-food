use ordering_food_organization_application::ApplicationError;
use ordering_food_organization_domain::{BrandId, OrganizationStatus, Store, StoreId};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const STORES_BRAND_SLUG_UNIQUE_CONSTRAINT: &str = "organization_stores_brand_slug_unique";

fn parse_store_id(store_id: &StoreId) -> Result<Uuid, ApplicationError> {
    Uuid::parse_str(store_id.as_str())
        .map_err(|_| ApplicationError::validation("store id must be a valid UUID"))
}

fn parse_brand_id(brand_id: &BrandId) -> Result<Uuid, ApplicationError> {
    Uuid::parse_str(brand_id.as_str())
        .map_err(|_| ApplicationError::validation("brand id must be a valid UUID"))
}

fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
    if error.as_database_error().is_some_and(|database_error| {
        database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
            && database_error.constraint() == Some(STORES_BRAND_SLUG_UNIQUE_CONSTRAINT)
    }) {
        ApplicationError::conflict("store slug already exists in brand")
    } else {
        ApplicationError::unexpected_with_source(message, error)
    }
}

pub(crate) async fn find_store_by_brand_slug(
    transaction: &mut Transaction<'static, Postgres>,
    brand_id: &BrandId,
    slug: &str,
) -> Result<Option<Store>, ApplicationError> {
    let brand_id = parse_brand_id(brand_id)?;
    let row = sqlx::query(
        r#"
        SELECT
            id,
            brand_id,
            slug,
            name,
            currency_code,
            timezone,
            status,
            created_at,
            updated_at,
            deleted_at
        FROM organization.stores
        WHERE brand_id = $1 AND slug = $2
        LIMIT 1
        "#,
    )
    .bind(brand_id)
    .bind(slug)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| {
        ApplicationError::unexpected_with_source(
            "failed to load organization store aggregate by brand slug",
            error,
        )
    })?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(Store::rehydrate(
        StoreId::new(row.get::<Uuid, _>("id").to_string()),
        BrandId::new(row.get::<Uuid, _>("brand_id").to_string()),
        row.get::<String, _>("slug"),
        row.get::<String, _>("name"),
        row.get::<String, _>("currency_code"),
        row.get::<String, _>("timezone"),
        OrganizationStatus::parse(row.get::<String, _>("status"))?,
        row.get("created_at"),
        row.get("updated_at"),
        row.get("deleted_at"),
    )?))
}

pub(crate) async fn insert_store(
    transaction: &mut Transaction<'static, Postgres>,
    store: &Store,
) -> Result<(), ApplicationError> {
    let store_id = parse_store_id(store.id())?;
    let brand_id = parse_brand_id(store.brand_id())?;
    sqlx::query(
        r#"
        INSERT INTO organization.stores (
            id,
            brand_id,
            slug,
            name,
            currency_code,
            timezone,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(store_id)
    .bind(brand_id)
    .bind(store.slug())
    .bind(store.name())
    .bind(store.currency_code())
    .bind(store.timezone())
    .bind(store.status().as_str())
    .bind(store.created_at())
    .bind(store.updated_at())
    .bind(store.deleted_at())
    .execute(&mut **transaction)
    .await
    .map_err(|error| map_write_error("failed to insert organization store", error))?;

    Ok(())
}

pub(crate) async fn update_store(
    transaction: &mut Transaction<'static, Postgres>,
    store: &Store,
) -> Result<(), ApplicationError> {
    let store_id = parse_store_id(store.id())?;
    let brand_id = parse_brand_id(store.brand_id())?;
    let result = sqlx::query(
        r#"
        UPDATE organization.stores
        SET
            brand_id = $2,
            slug = $3,
            name = $4,
            currency_code = $5,
            timezone = $6,
            status = $7,
            updated_at = $8,
            deleted_at = $9
        WHERE id = $1
        "#,
    )
    .bind(store_id)
    .bind(brand_id)
    .bind(store.slug())
    .bind(store.name())
    .bind(store.currency_code())
    .bind(store.timezone())
    .bind(store.status().as_str())
    .bind(store.updated_at())
    .bind(store.deleted_at())
    .execute(&mut **transaction)
    .await
    .map_err(|error| map_write_error("failed to update organization store", error))?;

    if result.rows_affected() == 0 {
        return Err(ApplicationError::not_found("store was not found"));
    }

    Ok(())
}
