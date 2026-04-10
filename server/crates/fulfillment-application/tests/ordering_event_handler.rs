#[path = "support/transaction.rs"]
mod transaction_support;

use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    ApplicationError, CommercialOrderCancelledByCustomer, CommercialOrderPlaced,
    CommercialOrderPlacedItem, CommercialOrderProjectionItemReadModel,
    CommercialOrderProjectionReadModel, CommercialOrderProjectionReadRepository,
    CommercialOrderProjectionStore, CommercialOrderStateChanged, IdGenerator,
    OrderingCommercialEventHandler, TransactionContext, WorkflowOrderRepository,
};
use ordering_food_fulfillment_domain::{FulfillmentOrder, FulfillmentOrderId, WorkflowStatus};
use ordering_food_shared_kernel::Timestamp;
use std::{
    sync::{Arc, Mutex},
};
use time::macros::datetime;
use transaction_support::RecordingTransactionManager;

struct FixedIdGenerator;

impl IdGenerator for FixedIdGenerator {
    fn next_fulfillment_order_id(&self) -> FulfillmentOrderId {
        FulfillmentOrderId::new("workflow-1")
    }
}

#[derive(Default)]
struct InMemoryCommercialProjectionRepository {
    projection: Mutex<Option<CommercialOrderProjectionReadModel>>,
}

#[async_trait]
impl CommercialOrderProjectionReadRepository for InMemoryCommercialProjectionRepository {
    async fn get_by_ordering_order_id(
        &self,
        _ordering_order_id: &str,
    ) -> Result<Option<CommercialOrderProjectionReadModel>, ApplicationError> {
        Ok(self.projection.lock().unwrap().clone())
    }
}

#[async_trait]
impl CommercialOrderProjectionStore for InMemoryCommercialProjectionRepository {
    async fn upsert(
        &self,
        _tx: &mut dyn TransactionContext,
        projection: &CommercialOrderProjectionReadModel,
    ) -> Result<(), ApplicationError> {
        *self.projection.lock().unwrap() = Some(projection.clone());
        Ok(())
    }

    async fn update_status(
        &self,
        _tx: &mut dyn TransactionContext,
        ordering_order_id: &str,
        status: &str,
        updated_at: Timestamp,
    ) -> Result<(), ApplicationError> {
        let mut guard = self.projection.lock().unwrap();
        let projection = guard
            .as_mut()
            .ok_or_else(|| ApplicationError::not_found("commercial projection was not found"))?;

        assert_eq!(projection.order_id, ordering_order_id);
        projection.status = status.to_string();
        projection.updated_at = updated_at;
        Ok(())
    }
}

#[derive(Default)]
struct InMemoryWorkflowRepository {
    order: Mutex<Option<FulfillmentOrder>>,
}

#[async_trait]
impl WorkflowOrderRepository for InMemoryWorkflowRepository {
    async fn find_by_ordering_order_id(
        &self,
        _tx: &mut dyn TransactionContext,
        _ordering_order_id: &str,
    ) -> Result<Option<FulfillmentOrder>, ApplicationError> {
        Ok(self.order.lock().unwrap().clone())
    }

    async fn insert(
        &self,
        _tx: &mut dyn TransactionContext,
        order: &FulfillmentOrder,
    ) -> Result<(), ApplicationError> {
        *self.order.lock().unwrap() = Some(order.clone());
        Ok(())
    }

    async fn update(
        &self,
        _tx: &mut dyn TransactionContext,
        order: &FulfillmentOrder,
    ) -> Result<(), ApplicationError> {
        *self.order.lock().unwrap() = Some(order.clone());
        Ok(())
    }
}

fn build_handler(
    commercial_projections: Arc<InMemoryCommercialProjectionRepository>,
    workflow_orders: Arc<InMemoryWorkflowRepository>,
    transactions: Arc<RecordingTransactionManager>,
) -> OrderingCommercialEventHandler {
    OrderingCommercialEventHandler::new(
        workflow_orders,
        commercial_projections,
        transactions,
        Arc::new(FixedIdGenerator),
    )
}

fn sample_order_placed() -> CommercialOrderPlaced {
    CommercialOrderPlaced {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        status: "placed".to_string(),
        subtotal_amount: 1800,
        total_amount: 1800,
        created_at: datetime!(2026-03-15 10:00 UTC),
        updated_at: datetime!(2026-03-15 10:00 UTC),
        items: vec![CommercialOrderPlacedItem {
            line_number: 1,
            catalog_item_id: "item-1".to_string(),
            name: "Noodles".to_string(),
            unit_price_amount: 1800,
            quantity: 1,
            line_total_amount: 1800,
        }],
    }
}

#[tokio::test]
async fn order_placed_event_bootstraps_projection_and_workflow() {
    let commercial_projections = Arc::new(InMemoryCommercialProjectionRepository::default());
    let workflow_orders = Arc::new(InMemoryWorkflowRepository::default());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let handler = build_handler(
        commercial_projections.clone(),
        workflow_orders.clone(),
        transactions.clone(),
    );

    handler
        .handle_order_placed(&sample_order_placed())
        .await
        .unwrap();

    let projection = commercial_projections
        .get_by_ordering_order_id("order-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        projection,
        CommercialOrderProjectionReadModel {
            order_id: "order-1".to_string(),
            customer_id: "customer-1".to_string(),
            store_id: "store-1".to_string(),
            status: "placed".to_string(),
            subtotal_amount: 1800,
            total_amount: 1800,
            created_at: datetime!(2026-03-15 10:00 UTC),
            updated_at: datetime!(2026-03-15 10:00 UTC),
            items: vec![CommercialOrderProjectionItemReadModel {
                line_number: 1,
                catalog_item_id: "item-1".to_string(),
                name: "Noodles".to_string(),
                unit_price_amount: 1800,
                quantity: 1,
                line_total_amount: 1800,
            }],
        }
    );
    assert_eq!(
        workflow_orders
            .order
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .status(),
        WorkflowStatus::PendingAcceptance
    );
    assert_eq!(transactions.began(), 1);
    assert_eq!(transactions.committed(), 1);
    assert_eq!(transactions.rolled_back(), 0);
}

#[tokio::test]
async fn commercial_state_changed_event_updates_local_projection_status() {
    let commercial_projections = Arc::new(InMemoryCommercialProjectionRepository::default());
    let workflow_orders = Arc::new(InMemoryWorkflowRepository::default());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let handler = build_handler(
        commercial_projections.clone(),
        workflow_orders,
        transactions.clone(),
    );
    handler
        .handle_order_placed(&sample_order_placed())
        .await
        .unwrap();

    handler
        .handle_order_commercial_state_changed(&CommercialOrderStateChanged {
            order_id: "order-1".to_string(),
            customer_id: "customer-1".to_string(),
            store_id: "store-1".to_string(),
            previous_status: "placed".to_string(),
            current_status: "cancelled_by_customer".to_string(),
            occurred_at: datetime!(2026-03-15 10:10 UTC),
        })
        .await
        .unwrap();

    let projection = commercial_projections
        .get_by_ordering_order_id("order-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(projection.status, "cancelled_by_customer");
    assert_eq!(projection.updated_at, datetime!(2026-03-15 10:10 UTC));
    assert_eq!(transactions.began(), 2);
    assert_eq!(transactions.committed(), 2);
    assert_eq!(transactions.rolled_back(), 0);
}

#[tokio::test]
async fn order_cancelled_by_customer_event_marks_workflow_cancelled() {
    let commercial_projections = Arc::new(InMemoryCommercialProjectionRepository::default());
    let workflow_orders = Arc::new(InMemoryWorkflowRepository::default());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let handler = build_handler(
        commercial_projections.clone(),
        workflow_orders.clone(),
        transactions.clone(),
    );
    handler
        .handle_order_placed(&sample_order_placed())
        .await
        .unwrap();

    handler
        .handle_order_cancelled_by_customer(&CommercialOrderCancelledByCustomer {
            order_id: "order-1".to_string(),
            customer_id: "customer-1".to_string(),
            store_id: "store-1".to_string(),
            occurred_at: datetime!(2026-03-15 10:15 UTC),
        })
        .await
        .unwrap();

    let projection = commercial_projections
        .get_by_ordering_order_id("order-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(projection.status, "cancelled_by_customer");
    assert_eq!(
        workflow_orders
            .order
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .status(),
        WorkflowStatus::CancelledByCustomer
    );
    assert_eq!(transactions.began(), 2);
    assert_eq!(transactions.committed(), 2);
    assert_eq!(transactions.rolled_back(), 0);
}
