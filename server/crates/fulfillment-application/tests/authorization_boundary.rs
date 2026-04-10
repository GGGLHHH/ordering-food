#[path = "support/fixed_clock.rs"]
mod fixed_clock;
#[path = "support/transaction.rs"]
mod transaction_support;

use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    AcceptOrder, AcceptOrderInput, ApplicationError, CommercialOrderProjectionQueryService,
    CommercialOrderProjectionReadModel, CommercialOrderProjectionReadRepository, CompleteOrder,
    CompleteOrderInput, MarkOrderReadyForPickup, MarkOrderReadyForPickupInput, RejectOrderByStore,
    RejectOrderByStoreInput, StartPreparingOrder, StartPreparingOrderInput, TransactionContext,
    WorkflowAction, WorkflowActionAuthorizer, WorkflowOrderRepository,
};
use ordering_food_fulfillment_domain::{FulfillmentOrder, WorkflowStatus};
use fixed_clock::FixedClock;
use std::{
    sync::{Arc, Mutex},
};
use time::macros::datetime;
use transaction_support::RecordingTransactionManager;

const WORKFLOW_ID: &str = "workflow-1";
const ORDER_ID: &str = "order-1";
const STORE_ID: &str = "store-1";
const CUSTOMER_ID: &str = "customer-1";
const UNAUTHORIZED_ACTOR_ID: &str = "merchant-unauthorized";

#[tokio::test]
async fn accept_order_denies_actor_without_store_access() {
    let fixture = TestFixture::for_status(WorkflowStatus::PendingAcceptance);
    let use_case = AcceptOrder::new(
        fixture.repository.clone(),
        fixture.authorizer.clone(),
        fixture.transactions.clone(),
        fixture.clock.clone(),
        fixture.commercial_queries.clone(),
    );

    let result = use_case
        .execute(AcceptOrderInput {
            order_id: ORDER_ID.to_string(),
            actor_user_id: UNAUTHORIZED_ACTOR_ID.to_string(),
        })
        .await;

    fixture.authorizer.assert_called_once_with(
        UNAUTHORIZED_ACTOR_ID,
        STORE_ID,
        WorkflowAction::Accept,
    );
    fixture.assert_authorizer_rejection_rolled_back_transaction();
    assert!(
        matches!(result, Err(ApplicationError::NotFound { .. })),
        "AcceptOrder should reject actors without store access"
    );
}

#[tokio::test]
async fn start_preparing_order_denies_actor_without_store_access() {
    let fixture = TestFixture::for_status(WorkflowStatus::Accepted);
    let use_case = StartPreparingOrder::new(
        fixture.repository.clone(),
        fixture.authorizer.clone(),
        fixture.transactions.clone(),
        fixture.clock.clone(),
        fixture.commercial_queries.clone(),
    );

    let result = use_case
        .execute(StartPreparingOrderInput {
            order_id: ORDER_ID.to_string(),
            actor_user_id: UNAUTHORIZED_ACTOR_ID.to_string(),
        })
        .await;

    fixture.authorizer.assert_called_once_with(
        UNAUTHORIZED_ACTOR_ID,
        STORE_ID,
        WorkflowAction::StartPreparing,
    );
    fixture.assert_authorizer_rejection_rolled_back_transaction();
    assert!(
        matches!(result, Err(ApplicationError::NotFound { .. })),
        "StartPreparingOrder should reject actors without store access"
    );
}

#[tokio::test]
async fn mark_order_ready_for_pickup_denies_actor_without_store_access() {
    let fixture = TestFixture::for_status(WorkflowStatus::Preparing);
    let use_case = MarkOrderReadyForPickup::new(
        fixture.repository.clone(),
        fixture.authorizer.clone(),
        fixture.transactions.clone(),
        fixture.clock.clone(),
        fixture.commercial_queries.clone(),
    );

    let result = use_case
        .execute(MarkOrderReadyForPickupInput {
            order_id: ORDER_ID.to_string(),
            actor_user_id: UNAUTHORIZED_ACTOR_ID.to_string(),
        })
        .await;

    fixture.authorizer.assert_called_once_with(
        UNAUTHORIZED_ACTOR_ID,
        STORE_ID,
        WorkflowAction::MarkReady,
    );
    fixture.assert_authorizer_rejection_rolled_back_transaction();
    assert!(
        matches!(result, Err(ApplicationError::NotFound { .. })),
        "MarkOrderReadyForPickup should reject actors without store access"
    );
}

#[tokio::test]
async fn complete_order_denies_actor_without_store_access() {
    let fixture = TestFixture::for_status(WorkflowStatus::ReadyForPickup);
    let use_case = CompleteOrder::new(
        fixture.repository.clone(),
        fixture.authorizer.clone(),
        fixture.transactions.clone(),
        fixture.clock.clone(),
        fixture.commercial_queries.clone(),
    );

    let result = use_case
        .execute(CompleteOrderInput {
            order_id: ORDER_ID.to_string(),
            actor_user_id: UNAUTHORIZED_ACTOR_ID.to_string(),
        })
        .await;

    fixture.authorizer.assert_called_once_with(
        UNAUTHORIZED_ACTOR_ID,
        STORE_ID,
        WorkflowAction::Complete,
    );
    fixture.assert_authorizer_rejection_rolled_back_transaction();
    assert!(
        matches!(result, Err(ApplicationError::NotFound { .. })),
        "CompleteOrder should reject actors without store access"
    );
}

#[tokio::test]
async fn reject_order_by_store_denies_actor_without_store_access() {
    let fixture = TestFixture::for_status(WorkflowStatus::Accepted);
    let use_case = RejectOrderByStore::new(
        fixture.repository.clone(),
        fixture.authorizer.clone(),
        fixture.transactions.clone(),
        fixture.clock.clone(),
        fixture.commercial_queries.clone(),
    );

    let result = use_case
        .execute(RejectOrderByStoreInput {
            order_id: ORDER_ID.to_string(),
            actor_user_id: UNAUTHORIZED_ACTOR_ID.to_string(),
        })
        .await;

    fixture.authorizer.assert_called_once_with(
        UNAUTHORIZED_ACTOR_ID,
        STORE_ID,
        WorkflowAction::Reject,
    );
    fixture.assert_authorizer_rejection_rolled_back_transaction();
    assert!(
        matches!(result, Err(ApplicationError::NotFound { .. })),
        "RejectOrderByStore should reject actors without store access"
    );
}

struct TestFixture {
    repository: Arc<InMemoryWorkflowRepository>,
    authorizer: Arc<RecordingWorkflowActionAuthorizer>,
    transactions: Arc<RecordingTransactionManager>,
    clock: Arc<FixedClock>,
    commercial_queries: Arc<CommercialOrderProjectionQueryService>,
}

impl TestFixture {
    fn for_status(status: WorkflowStatus) -> Self {
        let order = build_workflow_order(status);
        let order_id = order.ordering_order_id().to_string();
        let store_id = order.store_id().to_string();

        let projection = CommercialOrderProjectionReadModel {
            order_id: order_id.clone(),
            customer_id: CUSTOMER_ID.to_string(),
            store_id,
            status: "placed".to_string(),
            subtotal_amount: 1800,
            total_amount: 1800,
            created_at: datetime!(2026-04-01 09:00 UTC),
            updated_at: datetime!(2026-04-01 09:00 UTC),
            items: Vec::new(),
        };

        Self {
            repository: Arc::new(InMemoryWorkflowRepository::with_order(order)),
            authorizer: Arc::new(RecordingWorkflowActionAuthorizer::default()),
            transactions: Arc::new(RecordingTransactionManager::default()),
            clock: Arc::new(FixedClock {
                now: datetime!(2026-04-01 09:10 UTC),
            }),
            commercial_queries: Arc::new(CommercialOrderProjectionQueryService::new(Arc::new(
                AllowingCommercialProjectionRepository { projection },
            ))),
        }
    }

    fn assert_authorizer_rejection_rolled_back_transaction(&self) {
        assert_eq!(self.transactions.began(), 1);
        assert_eq!(self.transactions.committed(), 0);
        assert_eq!(self.transactions.rolled_back(), 1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthorizationCall {
    actor_user_id: String,
    store_id: String,
    action: WorkflowAction,
}

#[async_trait]
impl WorkflowActionAuthorizer for RecordingWorkflowActionAuthorizer {
    async fn ensure_actor_can_manage_order(
        &self,
        actor_user_id: &str,
        store_id: &str,
        action: WorkflowAction,
    ) -> Result<(), ApplicationError> {
        self.calls.lock().unwrap().push(AuthorizationCall {
            actor_user_id: actor_user_id.to_string(),
            store_id: store_id.to_string(),
            action,
        });
        Err(ApplicationError::not_found(
            "actor does not have permission to manage this order",
        ))
    }
}

#[derive(Default)]
struct RecordingWorkflowActionAuthorizer {
    calls: Mutex<Vec<AuthorizationCall>>,
}

impl RecordingWorkflowActionAuthorizer {
    fn assert_called_once_with(
        &self,
        expected_actor_user_id: &str,
        expected_store_id: &str,
        expected_action: WorkflowAction,
    ) {
        let calls = self.calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "authorizer should be called once");
        let call = &calls[0];
        assert_eq!(call.actor_user_id, expected_actor_user_id);
        assert_eq!(call.store_id, expected_store_id);
        assert_eq!(call.action, expected_action);
    }
}

fn build_workflow_order(status: WorkflowStatus) -> FulfillmentOrder {
    let mut order = FulfillmentOrder::bootstrap(
        WORKFLOW_ID,
        ORDER_ID,
        STORE_ID,
        datetime!(2026-04-01 09:00 UTC),
    );

    match status {
        WorkflowStatus::PendingAcceptance => {}
        WorkflowStatus::Accepted => {
            order.accept(datetime!(2026-04-01 09:01 UTC)).unwrap();
        }
        WorkflowStatus::Preparing => {
            order.accept(datetime!(2026-04-01 09:01 UTC)).unwrap();
            order
                .start_preparing(datetime!(2026-04-01 09:02 UTC))
                .unwrap();
        }
        WorkflowStatus::ReadyForPickup => {
            order.accept(datetime!(2026-04-01 09:01 UTC)).unwrap();
            order
                .start_preparing(datetime!(2026-04-01 09:02 UTC))
                .unwrap();
            order.mark_ready(datetime!(2026-04-01 09:03 UTC)).unwrap();
        }
        other => panic!("unsupported workflow status fixture: {:?}", other),
    }

    order
}

struct InMemoryWorkflowRepository {
    order: Mutex<Option<FulfillmentOrder>>,
}

impl InMemoryWorkflowRepository {
    fn with_order(order: FulfillmentOrder) -> Self {
        Self {
            order: Mutex::new(Some(order)),
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

struct AllowingCommercialProjectionRepository {
    projection: CommercialOrderProjectionReadModel,
}

#[async_trait]
impl CommercialOrderProjectionReadRepository for AllowingCommercialProjectionRepository {
    async fn get_by_ordering_order_id(
        &self,
        _ordering_order_id: &str,
    ) -> Result<Option<CommercialOrderProjectionReadModel>, ApplicationError> {
        Ok(Some(self.projection.clone()))
    }
}
