use crate::transaction::SqlxTransactionContext;
use ordering_food_ordering_application::{ApplicationError, TransactionContext};
use ordering_food_shared_kernel::Timestamp;
use serde_json::Value;
use sqlx::{Postgres, Transaction};

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxMessageWriteRequest {
    pub producer_context: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: Value,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Default, Clone)]
pub struct SqlxOutboxMessageAppender;

impl SqlxOutboxMessageAppender {
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

    pub async fn append(
        &self,
        tx: &mut dyn TransactionContext,
        request: OutboxMessageWriteRequest,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let occurred_at = request.occurred_at;

        sqlx::query(
            r#"
            INSERT INTO platform.outbox_messages (
                producer_context,
                event_type,
                aggregate_id,
                payload,
                occurred_at,
                available_at,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $5)
            "#,
        )
        .bind(request.producer_context)
        .bind(request.event_type)
        .bind(request.aggregate_id)
        .bind(request.payload)
        .bind(occurred_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to append outbox message", error)
        })?;

        Ok(())
    }
}
