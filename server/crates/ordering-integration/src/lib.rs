use ordering_food_ordering_application::{IdGenerator, OrderingModule};
use ordering_food_ordering_domain::OrderId;
use ordering_food_ordering_infrastructure_sqlx::build_ordering_module;
use ordering_food_platform_kernel::Clock;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

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
    OrderingContextRuntime {
        module: build_ordering_module(pg_pool, clock, Arc::new(UuidV4OrderIdGenerator)),
    }
}

struct UuidV4OrderIdGenerator;

impl IdGenerator for UuidV4OrderIdGenerator {
    fn next_order_id(&self) -> OrderId {
        OrderId::new(Uuid::new_v4().to_string())
    }
}
