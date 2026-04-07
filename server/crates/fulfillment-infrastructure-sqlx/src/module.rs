use crate::{
    SqlxCommercialOrderProjectionRepository, SqlxTransactionManager,
    SqlxWorkflowOrderReadRepository, SqlxWorkflowOrderRepository,
};
use ordering_food_fulfillment_application::{
    Clock, FulfillmentModule, IdGenerator, OrderingCommercialEventHandler,
};
use ordering_food_fulfillment_domain::FulfillmentOrderId;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub fn build_fulfillment_module(pool: PgPool, clock: Arc<dyn Clock>) -> Arc<FulfillmentModule> {
    let workflow_order_repository = Arc::new(SqlxWorkflowOrderRepository);
    let workflow_order_read_repository =
        Arc::new(SqlxWorkflowOrderReadRepository::new(pool.clone()));
    let commercial_order_projection_repository =
        Arc::new(SqlxCommercialOrderProjectionRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));
    let id_generator = Arc::new(UuidV4FulfillmentOrderIdGenerator);

    Arc::new(FulfillmentModule::new(
        workflow_order_repository,
        workflow_order_read_repository,
        commercial_order_projection_repository.clone(),
        commercial_order_projection_repository,
        transaction_manager,
        clock,
        id_generator,
    ))
}

pub fn build_ordering_commercial_event_handler(
    pool: PgPool,
    id_generator: Arc<dyn IdGenerator>,
) -> Arc<OrderingCommercialEventHandler> {
    let workflow_order_repository = Arc::new(SqlxWorkflowOrderRepository);
    let commercial_order_projection_repository =
        Arc::new(SqlxCommercialOrderProjectionRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));
    Arc::new(OrderingCommercialEventHandler::new(
        workflow_order_repository,
        commercial_order_projection_repository,
        transaction_manager,
        id_generator,
    ))
}

pub fn build_ordering_commercial_event_handler_with_uuid_ids(
    pool: PgPool,
) -> Arc<OrderingCommercialEventHandler> {
    build_ordering_commercial_event_handler(pool, Arc::new(UuidV4FulfillmentOrderIdGenerator))
}

struct UuidV4FulfillmentOrderIdGenerator;

impl IdGenerator for UuidV4FulfillmentOrderIdGenerator {
    fn next_fulfillment_order_id(&self) -> FulfillmentOrderId {
        FulfillmentOrderId::new(Uuid::now_v7().to_string())
    }
}
