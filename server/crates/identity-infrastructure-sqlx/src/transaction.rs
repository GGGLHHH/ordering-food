use async_trait::async_trait;
use ordering_food_identity_application::{
    ApplicationError, TransactionContext, TransactionManager,
};
use sqlx::{PgPool, Postgres, Transaction};
use std::any::Any;

pub struct SqlxTransactionContext {
    transaction: Transaction<'static, Postgres>,
}

impl SqlxTransactionContext {
    pub fn new(transaction: Transaction<'static, Postgres>) -> Self {
        Self { transaction }
    }

    pub fn transaction_mut(&mut self) -> &mut Transaction<'static, Postgres> {
        &mut self.transaction
    }

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self { transaction } = *self;
        transaction.commit().await.map_err(|error| {
            ApplicationError::unexpected_with_source("failed to commit identity transaction", error)
        })
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self { transaction } = *self;
        transaction.rollback().await.map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to rollback identity transaction",
                error,
            )
        })
    }
}

impl TransactionContext for SqlxTransactionContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

#[derive(Clone)]
pub struct SqlxTransactionManager {
    pool: PgPool,
}

impl SqlxTransactionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn downcast_context(
        tx: Box<dyn TransactionContext>,
    ) -> Result<Box<SqlxTransactionContext>, ApplicationError> {
        tx.into_any()
            .downcast::<SqlxTransactionContext>()
            .map_err(|_| {
                ApplicationError::unexpected("unexpected transaction context implementation")
            })
    }
}

#[async_trait]
impl TransactionManager for SqlxTransactionManager {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
        let transaction = self.pool.begin().await.map_err(|error| {
            ApplicationError::unexpected_with_source("failed to begin identity transaction", error)
        })?;

        Ok(Box::new(SqlxTransactionContext::new(transaction)))
    }

    async fn commit(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        Self::downcast_context(tx)?.commit().await
    }

    async fn rollback(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        Self::downcast_context(tx)?.rollback().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MIGRATOR;
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    fn database_url() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.env");
            let contents = fs::read_to_string(env_path).expect("read repository .env");
            contents
                .lines()
                .find_map(|line| line.strip_prefix("DATABASE__URL="))
                .expect("DATABASE__URL in .env")
                .trim()
                .to_string()
        })
    }

    async fn test_pool() -> PgPool {
        let pool = PgPool::connect(&database_url()).await.unwrap();
        MIGRATOR.run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn commit_persists_changes() {
        let pool = test_pool().await;
        let manager = SqlxTransactionManager::new(pool.clone());
        let user_id = Uuid::now_v7();

        let mut tx = manager.begin().await.unwrap();
        let transaction = tx
            .as_any_mut()
            .downcast_mut::<SqlxTransactionContext>()
            .unwrap()
            .transaction_mut();

        sqlx::query(
            "INSERT INTO identity.users (id, status, created_at, updated_at, deleted_at) VALUES ($1, 'active', NOW(), NOW(), NULL)",
        )
        .bind(user_id)
        .execute(&mut **transaction)
        .await
        .unwrap();

        manager.commit(tx).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity.users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        sqlx::query("DELETE FROM identity.users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rollback_discards_changes() {
        let pool = test_pool().await;
        let manager = SqlxTransactionManager::new(pool.clone());
        let user_id = Uuid::now_v7();

        let mut tx = manager.begin().await.unwrap();
        let transaction = tx
            .as_any_mut()
            .downcast_mut::<SqlxTransactionContext>()
            .unwrap()
            .transaction_mut();

        sqlx::query(
            "INSERT INTO identity.users (id, status, created_at, updated_at, deleted_at) VALUES ($1, 'active', NOW(), NOW(), NULL)",
        )
        .bind(user_id)
        .execute(&mut **transaction)
        .await
        .unwrap();

        manager.rollback(tx).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity.users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
