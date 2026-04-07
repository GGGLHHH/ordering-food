use ordering_food_identity_application::{ApplicationError, StoredCredential};
use ordering_food_identity_domain::UserId;
use ordering_food_shared_kernel::{Identifier, Timestamp};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub(crate) async fn find_credential_by_user_id(
    transaction: &mut Transaction<'static, Postgres>,
    user_id: &UserId,
) -> Result<Option<StoredCredential>, ApplicationError> {
    let user_id = parse_user_id(user_id)?;

    let row = sqlx::query(
        r#"
        SELECT user_id, password_hash, created_at, updated_at
        FROM identity.user_credentials
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| {
        ApplicationError::unexpected_with_source("failed to load user credential", error)
    })?;

    Ok(row.map(|row| StoredCredential {
        user_id: row.get::<Uuid, _>("user_id").to_string(),
        password_hash: row.get::<String, _>("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub(crate) async fn upsert_credential(
    transaction: &mut Transaction<'static, Postgres>,
    user_id: &UserId,
    password_hash: &str,
    now: Timestamp,
) -> Result<(), ApplicationError> {
    let user_id = parse_user_id(user_id)?;

    sqlx::query(
        r#"
        INSERT INTO identity.user_credentials (user_id, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $3)
        ON CONFLICT (user_id) DO UPDATE
        SET password_hash = EXCLUDED.password_hash, updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(user_id)
    .bind(password_hash)
    .bind(now)
    .execute(&mut **transaction)
    .await
    .map_err(|error| {
        ApplicationError::unexpected_with_source("failed to upsert user credential", error)
    })?;

    Ok(())
}

fn parse_user_id(user_id: &UserId) -> Result<Uuid, ApplicationError> {
    Uuid::parse_str(user_id.as_str())
        .map_err(|_| ApplicationError::validation("user id must be a valid UUID"))
}
