#[path = "support/fixed_clock.rs"]
mod fixed_clock;
#[path = "support/transaction.rs"]
mod transaction_support;

use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    AcceptOrder, AcceptOrderInput, ApplicationError, Clock, CommercialOrderProjectionQueryService,
    CommercialOrderProjectionReadModel, CommercialOrderProjectionReadRepository, CompleteOrder,
    CompleteOrderInput, MarkOrderReadyForPickup, MarkOrderReadyForPickupInput, RejectOrderByStore,
    RejectOrderByStoreInput, StartPreparingOrder, StartPreparingOrderInput, TransactionContext,
    WorkflowAction, WorkflowActionAuthorizer, WorkflowOrderRepository,
};
use fixed_clock::FixedClock;
use ordering_food_fulfillment_domain::FulfillmentOrder;
use std::{
    sync::{Arc, Mutex},
};
use time::macros::datetime;
use transaction_support::RecordingTransactionManager;

struct AllowingWorkflowActionAuthorizer;

#[async_trait]
impl WorkflowActionAuthorizer for AllowingWorkflowActionAuthorizer {
    async fn ensure_actor_can_manage_order(
        &self,
        _actor_user_id: &str,
        _store_id: &str,
        _action: WorkflowAction,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}

fn allowing_authorizer() -> Arc<dyn WorkflowActionAuthorizer> {
    Arc::new(AllowingWorkflowActionAuthorizer)
}

struct DenyingWorkflowActionAuthorizer;

#[async_trait]
impl WorkflowActionAuthorizer for DenyingWorkflowActionAuthorizer {
    async fn ensure_actor_can_manage_order(
        &self,
        _actor_user_id: &str,
        _store_id: &str,
        _action: WorkflowAction,
    ) -> Result<(), ApplicationError> {
        Err(ApplicationError::not_found(
            "actor does not have permission to manage this order",
        ))
    }
}

fn denying_authorizer() -> Arc<dyn WorkflowActionAuthorizer> {
    Arc::new(DenyingWorkflowActionAuthorizer)
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
    update_count: Mutex<u32>,
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
            update_count: Mutex::new(0),
        }
    }

    fn update_count(&self) -> u32 {
        *self.update_count.lock().unwrap()
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
        *self.update_count.lock().unwrap() += 1;
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

fn allowed_commercial_queries() -> Arc<CommercialOrderProjectionQueryService> {
    Arc::new(CommercialOrderProjectionQueryService::new(Arc::new(
        FixedCommercialProjectionRepository {
            projection: CommercialOrderProjectionReadModel {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                status: "placed".to_string(),
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
        allowing_authorizer(),
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
async fn accept_order_rolls_back_when_authorizer_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = AcceptOrder::new(
        repository.clone(),
        denying_authorizer(),
        transactions.clone(),
        fixed_clock(),
        allowed_commercial_queries(),
    );

    let error = use_case
        .execute(AcceptOrderInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
    assert_eq!(transactions.began(), 1);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 1);
    assert_eq!(repository.update_count(), 0);
}

#[tokio::test]
async fn start_preparing_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = StartPreparingOrder::new(
        repository,
        allowing_authorizer(),
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
async fn start_preparing_rolls_back_when_authorizer_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = StartPreparingOrder::new(
        repository.clone(),
        denying_authorizer(),
        transactions.clone(),
        fixed_clock(),
        allowed_commercial_queries(),
    );

    let error = use_case
        .execute(StartPreparingOrderInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
    assert_eq!(transactions.began(), 1);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 1);
    assert_eq!(repository.update_count(), 0);
}

#[tokio::test]
async fn mark_ready_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = MarkOrderReadyForPickup::new(
        repository,
        allowing_authorizer(),
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
async fn mark_ready_rolls_back_when_authorizer_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = MarkOrderReadyForPickup::new(
        repository.clone(),
        denying_authorizer(),
        transactions.clone(),
        fixed_clock(),
        allowed_commercial_queries(),
    );

    let error = use_case
        .execute(MarkOrderReadyForPickupInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
    assert_eq!(transactions.began(), 1);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 1);
    assert_eq!(repository.update_count(), 0);
}

#[tokio::test]
async fn complete_order_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = CompleteOrder::new(
        repository,
        allowing_authorizer(),
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
async fn complete_order_rolls_back_when_authorizer_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = CompleteOrder::new(
        repository.clone(),
        denying_authorizer(),
        transactions.clone(),
        fixed_clock(),
        allowed_commercial_queries(),
    );

    let error = use_case
        .execute(CompleteOrderInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
    assert_eq!(transactions.began(), 1);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 1);
    assert_eq!(repository.update_count(), 0);
}

#[tokio::test]
async fn reject_order_does_not_open_transaction_when_projection_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = RejectOrderByStore::new(
        repository,
        allowing_authorizer(),
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

#[tokio::test]
async fn reject_order_rolls_back_when_authorizer_rejects() {
    let repository = Arc::new(InMemoryWorkflowRepository::with_bootstrapped_order());
    let transactions = Arc::new(RecordingTransactionManager::default());
    let use_case = RejectOrderByStore::new(
        repository.clone(),
        denying_authorizer(),
        transactions.clone(),
        fixed_clock(),
        allowed_commercial_queries(),
    );

    let error = use_case
        .execute(RejectOrderByStoreInput {
            order_id: "order-1".to_string(),
            actor_user_id: "merchant-1".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
    assert_eq!(transactions.began(), 1);
    assert_eq!(transactions.committed(), 0);
    assert_eq!(transactions.rolled_back(), 1);
    assert_eq!(repository.update_count(), 0);
}
