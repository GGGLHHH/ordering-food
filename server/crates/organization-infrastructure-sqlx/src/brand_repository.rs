use ordering_food_organization_application::ApplicationError;
use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const BRANDS_SLUG_UNIQUE_CONSTRAINT: &str = "organization_brands_slug_unique";

fn parse_brand_id(brand_id: &BrandId) -> Result<Uuid, ApplicationError> {
    Uuid::parse_str(brand_id.as_str())
        .map_err(|_| ApplicationError::validation("brand id must be a valid UUID"))
}

fn map_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
    if error.as_database_error().is_some_and(|database_error| {
        database_error.code().as_deref() == Some(UNIQUE_VIOLATION_SQLSTATE)
            && database_error.constraint() == Some(BRANDS_SLUG_UNIQUE_CONSTRAINT)
    }) {
        ApplicationError::conflict("brand slug already exists")
    } else {
        ApplicationError::unexpected_with_source(message, error)
    }
}

pub(crate) async fn find_brand_by_id(
    transaction: &mut Transaction<'static, Postgres>,
    brand_id: &BrandId,
) -> Result<Option<Brand>, ApplicationError> {
    let brand_id = parse_brand_id(brand_id)?;
    let row = sqlx::query(
        r#"
        SELECT
            id,
            slug,
            name,
            status,
            created_at,
            updated_at,
            deleted_at
        FROM organization.brands
        WHERE id = $1
        LIMIT 1
        "#,
    )
    .bind(brand_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| {
        ApplicationError::unexpected_with_source(
            "failed to load organization brand aggregate",
            error,
        )
    })?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(Brand::rehydrate(
        BrandId::new(row.get::<Uuid, _>("id").to_string()),
        row.get::<String, _>("slug"),
        row.get::<String, _>("name"),
        OrganizationStatus::parse(row.get::<String, _>("status"))?,
        row.get("created_at"),
        row.get("updated_at"),
        row.get("deleted_at"),
    )?))
}

pub(crate) async fn insert_brand(
    transaction: &mut Transaction<'static, Postgres>,
    brand: &Brand,
) -> Result<(), ApplicationError> {
    let brand_id = parse_brand_id(brand.id())?;
    sqlx::query(
        r#"
        INSERT INTO organization.brands (
            id,
            slug,
            name,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(brand_id)
    .bind(brand.slug())
    .bind(brand.name())
    .bind(brand.status().as_str())
    .bind(brand.created_at())
    .bind(brand.updated_at())
    .bind(brand.deleted_at())
    .execute(&mut **transaction)
    .await
    .map_err(|error| map_write_error("failed to insert organization brand", error))?;

    Ok(())
}
