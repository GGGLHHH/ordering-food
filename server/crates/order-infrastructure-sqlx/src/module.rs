use crate::{SqlxOrderReadRepository, SqlxOrderRepository, SqlxTransactionManager};
use ordering_food_order_application::{Clock, IdGenerator, OrderModule};
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_order_module(
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
) -> Arc<OrderModule> {
    let order_repository = Arc::new(SqlxOrderRepository);
    let order_read_repository = Arc::new(SqlxOrderReadRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));

    Arc::new(OrderModule::new(
        order_repository,
        order_read_repository,
        transaction_manager,
        clock,
        id_generator,
    ))
}
