use crate::{
    AcceptOrder, Clock, CommercialOrderProjectionQueryService,
    CommercialOrderProjectionReadRepository, CommercialOrderProjectionStore, CompleteOrder,
    IdGenerator, MarkOrderReadyForPickup, OrderingCommercialEventHandler, RejectOrderByStore,
    StartPreparingOrder, TransactionManager, WorkflowActionAuthorizer, WorkflowOrderQueryService,
    WorkflowOrderReadRepository, WorkflowOrderRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct FulfillmentModule {
    accept_order: Arc<AcceptOrder>,
    start_preparing_order: Arc<StartPreparingOrder>,
    mark_order_ready_for_pickup: Arc<MarkOrderReadyForPickup>,
    complete_order: Arc<CompleteOrder>,
    reject_order_by_store: Arc<RejectOrderByStore>,
    workflow_queries: Arc<WorkflowOrderQueryService>,
    commercial_queries: Arc<CommercialOrderProjectionQueryService>,
    ordering_event_handler: Arc<OrderingCommercialEventHandler>,
}

impl FulfillmentModule {
    pub fn new(
        workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
        workflow_order_read_repository: Arc<dyn WorkflowOrderReadRepository>,
        commercial_order_projection_read_repository: Arc<
            dyn CommercialOrderProjectionReadRepository,
        >,
        commercial_order_projection_store: Arc<dyn CommercialOrderProjectionStore>,
        workflow_action_authorizer: Arc<dyn WorkflowActionAuthorizer>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        let commercial_queries = Arc::new(CommercialOrderProjectionQueryService::new(
            commercial_order_projection_read_repository,
        ));
        let ordering_event_handler = Arc::new(OrderingCommercialEventHandler::new(
            workflow_order_repository.clone(),
            commercial_order_projection_store,
            transaction_manager.clone(),
            id_generator,
        ));

        Self {
            accept_order: Arc::new(AcceptOrder::new(
                workflow_order_repository.clone(),
                workflow_action_authorizer.clone(),
                transaction_manager.clone(),
                clock.clone(),
                commercial_queries.clone(),
            )),
            start_preparing_order: Arc::new(StartPreparingOrder::new(
                workflow_order_repository.clone(),
                workflow_action_authorizer.clone(),
                transaction_manager.clone(),
                clock.clone(),
                commercial_queries.clone(),
            )),
            mark_order_ready_for_pickup: Arc::new(MarkOrderReadyForPickup::new(
                workflow_order_repository.clone(),
                workflow_action_authorizer.clone(),
                transaction_manager.clone(),
                clock.clone(),
                commercial_queries.clone(),
            )),
            complete_order: Arc::new(CompleteOrder::new(
                workflow_order_repository.clone(),
                workflow_action_authorizer.clone(),
                transaction_manager.clone(),
                clock.clone(),
                commercial_queries.clone(),
            )),
            reject_order_by_store: Arc::new(RejectOrderByStore::new(
                workflow_order_repository,
                workflow_action_authorizer,
                transaction_manager,
                clock,
                commercial_queries.clone(),
            )),
            workflow_queries: Arc::new(WorkflowOrderQueryService::new(
                workflow_order_read_repository,
            )),
            commercial_queries,
            ordering_event_handler,
        }
    }

    pub fn accept_order(&self) -> &Arc<AcceptOrder> {
        &self.accept_order
    }

    pub fn start_preparing_order(&self) -> &Arc<StartPreparingOrder> {
        &self.start_preparing_order
    }

    pub fn mark_order_ready_for_pickup(&self) -> &Arc<MarkOrderReadyForPickup> {
        &self.mark_order_ready_for_pickup
    }

    pub fn complete_order(&self) -> &Arc<CompleteOrder> {
        &self.complete_order
    }

    pub fn reject_order_by_store(&self) -> &Arc<RejectOrderByStore> {
        &self.reject_order_by_store
    }

    pub fn workflow_queries(&self) -> &Arc<WorkflowOrderQueryService> {
        &self.workflow_queries
    }

    pub fn commercial_queries(&self) -> &Arc<CommercialOrderProjectionQueryService> {
        &self.commercial_queries
    }

    pub fn ordering_event_handler(&self) -> &Arc<OrderingCommercialEventHandler> {
        &self.ordering_event_handler
    }
}
