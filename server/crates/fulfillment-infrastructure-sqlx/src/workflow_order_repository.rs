use crate::{db_workflow_status::DbWorkflowStatus, transaction::SqlxTransactionContext};
use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    ApplicationError, TransactionContext, WorkflowOrderRepository,
};
use ordering_food_fulfillment_domain::{FulfillmentOrder, FulfillmentOrderId, WorkflowStatus};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct SqlxWorkflowOrderRepository;

impl SqlxWorkflowOrderRepository {
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

    fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(value)
            .map_err(|_| ApplicationError::validation(format!("{field} must be a valid UUID")))
    }
}

#[async_trait]
impl WorkflowOrderRepository for SqlxWorkflowOrderRepository {
    async fn find_by_ordering_order_id(
        &self,
        tx: &mut dyn TransactionContext,
        ordering_order_id: &str,
    ) -> Result<Option<FulfillmentOrder>, ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let ordering_order_id = Self::parse_uuid(ordering_order_id, "ordering order id")?;

        let row = sqlx::query(
            r#"
            SELECT
                id,
                ordering_order_id,
                store_id,
                status,
                created_at,
                updated_at
            FROM fulfillment.workflow_orders
            WHERE ordering_order_id = $1
            LIMIT 1
            "#,
        )
        .bind(ordering_order_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load fulfillment workflow order",
                error,
            )
        })?;

        Ok(row.map(|row| {
            FulfillmentOrder::rehydrate(
                FulfillmentOrderId::new(row.get::<Uuid, _>("id").to_string()),
                row.get::<Uuid, _>("ordering_order_id").to_string(),
                row.get::<Uuid, _>("store_id").to_string(),
                WorkflowStatus::from(row.get::<DbWorkflowStatus, _>("status")),
                row.get("created_at"),
                row.get("updated_at"),
            )
        }))
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        order: &FulfillmentOrder,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let fulfillment_order_id = Self::parse_uuid(order.id().as_str(), "fulfillment order id")?;
        let ordering_order_id = Self::parse_uuid(order.ordering_order_id(), "ordering order id")?;
        let store_id = Self::parse_uuid(order.store_id(), "store id")?;

        sqlx::query(
            r#"
            INSERT INTO fulfillment.workflow_orders (
                id,
                ordering_order_id,
                store_id,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(fulfillment_order_id)
        .bind(ordering_order_id)
        .bind(store_id)
        .bind(DbWorkflowStatus::from(order.status()))
        .bind(order.created_at())
        .bind(order.updated_at())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to insert fulfillment workflow order",
                error,
            )
        })?;

        Ok(())
    }

    async fn update(
        &self,
        tx: &mut dyn TransactionContext,
        order: &FulfillmentOrder,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let ordering_order_id = Self::parse_uuid(order.ordering_order_id(), "ordering order id")?;

        sqlx::query(
            r#"
            UPDATE fulfillment.workflow_orders
            SET status = $2, updated_at = $3
            WHERE ordering_order_id = $1
            "#,
        )
        .bind(ordering_order_id)
        .bind(DbWorkflowStatus::from(order.status()))
        .bind(order.updated_at())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to update fulfillment workflow order",
                error,
            )
        })?;

        Ok(())
    }
}
