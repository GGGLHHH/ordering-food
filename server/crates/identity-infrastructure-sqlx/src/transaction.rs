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
