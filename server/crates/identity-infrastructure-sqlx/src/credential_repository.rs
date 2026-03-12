use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_identity_application::{
    ApplicationError, CredentialRepository, StoredCredential, TransactionContext,
};
use ordering_food_identity_domain::UserId;
use ordering_food_shared_kernel::{Identifier, Timestamp};
use sqlx::{Postgres, Row, Transaction};

#[derive(Debug, Default)]
pub struct SqlxCredentialRepository;

impl SqlxCredentialRepository {
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
}

#[async_trait]
impl CredentialRepository for SqlxCredentialRepository {
    async fn find_by_user_id(
        &self,
        tx: &mut dyn TransactionContext,
        user_id: &UserId,
    ) -> Result<Option<StoredCredential>, ApplicationError> {
        let transaction = Self::transaction(tx)?;

        let row = sqlx::query(
            r#"
            SELECT user_id, password_hash, created_at, updated_at
            FROM identity.user_credentials
            WHERE user_id = $1
            "#,
        )
        .bind(user_id.as_str())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to load user credential", error)
        })?;

        Ok(row.map(|r| StoredCredential {
            user_id: r.get::<String, _>("user_id"),
            password_hash: r.get::<String, _>("password_hash"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn upsert(
        &self,
        tx: &mut dyn TransactionContext,
        user_id: &UserId,
        password_hash: &str,
        now: Timestamp,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;

        sqlx::query(
            r#"
            INSERT INTO identity.user_credentials (user_id, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $3)
            ON CONFLICT (user_id) DO UPDATE
            SET password_hash = EXCLUDED.password_hash, updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(user_id.as_str())
        .bind(password_hash)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to upsert user credential", error)
        })?;

        Ok(())
    }
}
