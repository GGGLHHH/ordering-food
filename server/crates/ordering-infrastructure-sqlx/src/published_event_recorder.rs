use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_ordering_application::{
    ApplicationError, OrderCancelledByCustomer, OrderCommercialStateChanged, OrderPlaced,
    OrderingPublishedEventRecorder, TransactionContext,
};
use ordering_food_shared_kernel::Timestamp;
use serde_json::Value;
use sqlx::{Postgres, Transaction};

#[derive(Debug, Default)]
pub struct SqlxPublishedEventRecorder;

impl SqlxPublishedEventRecorder {
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

    async fn insert_event(
        tx: &mut dyn TransactionContext,
        event_type: &'static str,
        aggregate_id: &str,
        payload: Value,
        occurred_at: Timestamp,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
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
        .bind("ordering")
        .bind(event_type)
        .bind(aggregate_id)
        .bind(payload)
        .bind(occurred_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to record ordering published event",
                error,
            )
        })?;

        Ok(())
    }
}

#[async_trait]
impl OrderingPublishedEventRecorder for SqlxPublishedEventRecorder {
    async fn record_order_placed(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderPlaced,
    ) -> Result<(), ApplicationError> {
        Self::insert_event(
            tx,
            "ordering.order_placed",
            &event.order_id,
            serde_json::to_value(event).map_err(|error| {
                ApplicationError::unexpected_with_source(
                    "failed to serialize order placed event",
                    error,
                )
            })?,
            event.updated_at,
        )
        .await
    }

    async fn record_order_commercial_state_changed(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderCommercialStateChanged,
    ) -> Result<(), ApplicationError> {
        Self::insert_event(
            tx,
            "ordering.order_commercial_state_changed",
            &event.order_id,
            serde_json::to_value(event).map_err(|error| {
                ApplicationError::unexpected_with_source(
                    "failed to serialize order commercial state event",
                    error,
                )
            })?,
            event.occurred_at,
        )
        .await
    }

    async fn record_order_cancelled_by_customer(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderCancelledByCustomer,
    ) -> Result<(), ApplicationError> {
        Self::insert_event(
            tx,
            "ordering.order_cancelled_by_customer",
            &event.order_id,
            serde_json::to_value(event).map_err(|error| {
                ApplicationError::unexpected_with_source(
                    "failed to serialize customer cancellation event",
                    error,
                )
            })?,
            event.occurred_at,
        )
        .await
    }
}
