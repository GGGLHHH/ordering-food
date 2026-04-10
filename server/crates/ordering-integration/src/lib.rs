pub mod published_event_adapter;

use ordering_food_ordering_application::{IdGenerator, OrderingModule};
use ordering_food_ordering_domain::OrderId;
use ordering_food_ordering_infrastructure_sqlx::build_ordering_sqlx_components;
use ordering_food_platform_kernel::Clock;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::published_event_adapter::AdapterBackedOrderingEventRecorder;

#[derive(Clone)]
pub struct OrderingContextRuntime {
    module: Arc<OrderingModule>,
}

impl OrderingContextRuntime {
    pub fn module(&self) -> &Arc<OrderingModule> {
        &self.module
    }
}

pub fn build_ordering_context_runtime(
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
) -> OrderingContextRuntime {
    let sqlx_components = build_ordering_sqlx_components(pg_pool);
    let event_recorder = Arc::new(AdapterBackedOrderingEventRecorder::new(
        sqlx_components.outbox_message_appender,
    ));

    OrderingContextRuntime {
        module: Arc::new(OrderingModule::new(
            sqlx_components.order_repository,
            sqlx_components.order_read_repository,
            sqlx_components.transaction_manager,
            clock,
            Arc::new(UuidV7OrderIdGenerator),
            event_recorder,
        )),
    }
}

struct UuidV7OrderIdGenerator;

impl IdGenerator for UuidV7OrderIdGenerator {
    fn next_order_id(&self) -> OrderId {
        OrderId::new(Uuid::now_v7().to_string())
    }
}
