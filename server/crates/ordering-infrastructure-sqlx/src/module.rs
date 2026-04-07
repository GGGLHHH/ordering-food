use crate::{
    SqlxOrderReadRepository, SqlxOrderRepository, SqlxPublishedEventRecorder,
    SqlxTransactionManager,
};
use ordering_food_ordering_application::{Clock, IdGenerator, OrderingModule};
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_ordering_module(
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
) -> Arc<OrderingModule> {
    let order_repository = Arc::new(SqlxOrderRepository);
    let order_read_repository = Arc::new(SqlxOrderReadRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));
    let event_recorder = Arc::new(SqlxPublishedEventRecorder);

    Arc::new(OrderingModule::new(
        order_repository,
        order_read_repository,
        transaction_manager,
        clock,
        id_generator,
        event_recorder,
    ))
}
