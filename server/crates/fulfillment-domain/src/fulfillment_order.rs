use crate::{FulfillmentOrderId, WorkflowStatus};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FulfillmentOrder {
    id: FulfillmentOrderId,
    ordering_order_id: String,
    store_id: String,
    status: WorkflowStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl FulfillmentOrder {
    pub fn bootstrap(
        fulfillment_order_id: impl Into<String>,
        ordering_order_id: impl Into<String>,
        store_id: impl Into<String>,
        now: Timestamp,
    ) -> Self {
        let fulfillment_order_id = fulfillment_order_id.into();
        Self {
            id: FulfillmentOrderId::new(fulfillment_order_id),
            ordering_order_id: ordering_order_id.into(),
            store_id: store_id.into(),
            status: WorkflowStatus::PendingAcceptance,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn rehydrate(
        id: FulfillmentOrderId,
        ordering_order_id: impl Into<String>,
        store_id: impl Into<String>,
        status: WorkflowStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            ordering_order_id: ordering_order_id.into(),
            store_id: store_id.into(),
            status,
            created_at,
            updated_at,
        }
    }

    pub fn accept(&mut self, now: Timestamp) -> Result<(), crate::DomainError> {
        self.status = self.status.accept()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn start_preparing(&mut self, now: Timestamp) -> Result<(), crate::DomainError> {
        self.status = self.status.start_preparing()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn mark_ready(&mut self, now: Timestamp) -> Result<(), crate::DomainError> {
        self.status = self.status.mark_ready()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn complete(&mut self, now: Timestamp) -> Result<(), crate::DomainError> {
        self.status = self.status.complete()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn cancel_by_customer(&mut self, now: Timestamp) -> Result<(), crate::DomainError> {
        self.status = self.status.cancel_by_customer()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn reject_by_store(&mut self, now: Timestamp) -> Result<(), crate::DomainError> {
        self.status = self.status.reject_by_store()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn id(&self) -> &FulfillmentOrderId {
        &self.id
    }

    pub fn ordering_order_id(&self) -> &str {
        &self.ordering_order_id
    }

    pub fn store_id(&self) -> &str {
        &self.store_id
    }

    pub fn status(&self) -> WorkflowStatus {
        self.status
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn bootstrap_preserves_distinct_fulfillment_identity() {
        let order = FulfillmentOrder::bootstrap(
            "workflow-1",
            "order-1",
            "store-1",
            datetime!(2026-03-15 10:00 UTC),
        );

        assert_eq!(order.id().as_str(), "workflow-1");
        assert_eq!(order.ordering_order_id(), "order-1");
        assert_eq!(order.status(), WorkflowStatus::PendingAcceptance);
    }

    #[test]
    fn customer_cancellation_is_allowed_before_preparing() {
        let mut order = FulfillmentOrder::bootstrap(
            "workflow-1",
            "order-1",
            "store-1",
            datetime!(2026-03-15 10:00 UTC),
        );
        order.accept(datetime!(2026-03-15 10:01 UTC)).unwrap();

        order
            .cancel_by_customer(datetime!(2026-03-15 10:02 UTC))
            .unwrap();

        assert_eq!(order.status(), WorkflowStatus::CancelledByCustomer);
    }

    #[test]
    fn workflow_cannot_complete_before_ready() {
        let mut order = FulfillmentOrder::bootstrap(
            "workflow-1",
            "order-1",
            "store-1",
            datetime!(2026-03-15 10:00 UTC),
        );

        let error = order.complete(datetime!(2026-03-15 10:01 UTC)).unwrap_err();

        assert_eq!(
            error,
            crate::DomainError::InvalidTransition {
                event: "complete".to_string(),
                status: "pending_acceptance".to_string(),
            }
        );
    }
}
