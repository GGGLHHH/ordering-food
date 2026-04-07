use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    AcceptOrder, AcceptOrderInput, ApplicationError, Clock, CommercialOrderProjectionQueryService,
    CommercialOrderProjectionReadModel, CommercialOrderProjectionReadRepository, CompleteOrder,
    CompleteOrderInput, MarkOrderReadyForPickup, MarkOrderReadyForPickupInput, RejectOrderByStore,
    RejectOrderByStoreInput, StartPreparingOrder, StartPreparingOrderInput, TransactionContext,
    TransactionManager, WorkflowOrderRepository,
};
use ordering_food_fulfillment_domain::FulfillmentOrder;
use ordering_food_shared_kernel::Timestamp;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use time::macros::datetime;

#[derive(Default)]
struct FakeTransactionContext;

impl TransactionContext for FakeTransactionContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

#[derive(Default)]
struct RecordingTransactionManager {
    began: Mutex<u32>,
    committed: Mutex<u32>,
    rolled_back: Mutex<u32>,
}

impl RecordingTransactionManager {
    fn began(&self) -> u32 {
        *self.began.lock().unwrap()
    }

    fn committed(&self) -> u32 {
        *self.committed.lock().unwrap()
    }

    fn rolled_back(&self) -> u32 {
        *self.rolled_back.lock().unwrap()
    }
}

#[async_trait]
impl TransactionManager for RecordingTransactionManager {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
        *self.began.lock().unwrap() += 1;
        Ok(Box::new(FakeTransactionContext))
    }

    async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        *self.committed.lock().unwrap() += 1;
        Ok(())
    }

    async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        *self.rolled_back.lock().unwrap() += 1;
        Ok(())
    }
}

struct FixedClock {
    now: Timestamp,
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

struct FixedCommercialProjectionRepository {
    projection: CommercialOrderProjectionReadModel,
}

#[async_trait]
impl CommercialOrderProjectionReadRepository for FixedCommercialProjectionRepository {
    async fn get_by_ordering_order_id(
        &self,
        _ordering_order_id: &str,
    ) -> Result<Option<CommercialOrderProjectionReadModel>, ApplicationError> {
        Ok(Some(self.projection.clone()))
    }
}

struct InMemoryWorkflowRepository {
    order: Mutex<Option<FulfillmentOrder>>,
}

impl InMemoryWorkflowRepository {
    fn with_bootstrapped_order() -> Self {
        Self {
            order: Mutex::new(Some(FulfillmentOrder::bootstrap(
                "workflow-1",
                "order-1",
                "store-1",
                datetime!(2026-03-15 09:00 UTC),
            ))),
        }
    }
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

fn fixed_clock() -> Arc<dyn Clock> {
    Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:00 UTC),
    })
}

fn blocked_commercial_queries() -> Arc<CommercialOrderProjectionQueryService> {
    Arc::new(CommercialOrderProjectionQueryService::new(Arc::new(
        FixedCommercialProjectionRepository {
            projection: CommercialOrderProjectionReadModel {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                status: "cancelled_by_customer".to_string(),
                subtotal_amount: 1800,
                total_amount: 1800,
                created_at: datetime!(2026-03-15 09:00 UTC),
                updated_at: datetime!(2026-03-15 09:30 UTC),
                items: Vec::new(),
            },
        },
    )))
}

#[tokio::test]
async fn accept_order_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = AcceptOrder::new(
        repository,
        transactions.clone(),
        fixed_clock(),
        blocked_commercial_queries(),
    );

    let error = use_case
        .execute(AcceptOrderInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Conflict { .. }));
    assert_eq!(transactions.began(), 0);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 0);
}

#[tokio::test]
async fn start_preparing_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = StartPreparingOrder::new(
        repository,
        transactions.clone(),
        fixed_clock(),
        blocked_commercial_queries(),
    );

    let error = use_case
        .execute(StartPreparingOrderInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Conflict { .. }));
    assert_eq!(transactions.began(), 0);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 0);
}

#[tokio::test]
async fn mark_ready_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = MarkOrderReadyForPickup::new(
        repository,
        transactions.clone(),
        fixed_clock(),
        blocked_commercial_queries(),
    );

    let error = use_case
        .execute(MarkOrderReadyForPickupInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Conflict { .. }));
    assert_eq!(transactions.began(), 0);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 0);
}

#[tokio::test]
async fn complete_order_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = CompleteOrder::new(
        repository,
        transactions.clone(),
        fixed_clock(),
        blocked_commercial_queries(),
    );

    let error = use_case
        .execute(CompleteOrderInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Conflict { .. }));
    assert_eq!(transactions.began(), 0);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 0);
}

#[tokio::test]
async fn reject_order_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = RejectOrderByStore::new(
        repository,
        transactions.clone(),
        fixed_clock(),
        blocked_commercial_queries(),
    );

    let error = use_case
        .execute(RejectOrderByStoreInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Conflict { .. }));
    assert_eq!(transactions.began(), 0);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 0);
}
