use crate::db_workflow_status::DbWorkflowStatus;
use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    ApplicationError, WorkflowOrderReadModel, WorkflowOrderReadRepository,
};
use ordering_food_shared_kernel::Timestamp;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxWorkflowOrderReadRepository {
    pool: PgPool,
}

impl SqlxWorkflowOrderReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkflowOrderReadRepository for SqlxWorkflowOrderReadRepository {
    async fn get_by_ordering_order_id(
        &self,
        ordering_order_id: &str,
    ) -> Result<Option<WorkflowOrderReadModel>, ApplicationError> {
        let ordering_order_id = Uuid::parse_str(ordering_order_id)
            .map_err(|_| ApplicationError::validation("ordering order id must be a valid UUID"))?;

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
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query fulfillment workflow order",
                error,
            )
        })?;

        Ok(row.map(|row| WorkflowOrderReadModel {
            fulfillment_order_id: row.get::<Uuid, _>("id").to_string(),
            ordering_order_id: row.get::<Uuid, _>("ordering_order_id").to_string(),
            store_id: row.get::<Uuid, _>("store_id").to_string(),
            status: to_status_string(row.get::<DbWorkflowStatus, _>("status")),
            created_at: row.get::<Timestamp, _>("created_at"),
            updated_at: row.get::<Timestamp, _>("updated_at"),
        }))
    }
}

fn to_status_string(value: DbWorkflowStatus) -> String {
    match value {
        DbWorkflowStatus::PendingAcceptance => "pending_acceptance",
        DbWorkflowStatus::Accepted => "accepted",
        DbWorkflowStatus::Preparing => "preparing",
        DbWorkflowStatus::ReadyForPickup => "ready_for_pickup",
        DbWorkflowStatus::Completed => "completed",
        DbWorkflowStatus::CancelledByCustomer => "cancelled_by_customer",
        DbWorkflowStatus::RejectedByStore => "rejected_by_store",
    }
    .to_string()
}
