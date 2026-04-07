use crate::{
    composition::capabilities::ApiCapabilityRegistry, config::Settings, runtime::SystemClock,
};
use ordering_food_platform_kernel::Clock;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiPlatform {
    pub settings: Settings,
    pub pg_pool: PgPool,
    pub redis_client: redis::Client,
    pub clock: Arc<dyn Clock>,
    pub capabilities: Arc<ApiCapabilityRegistry>,
}

impl ApiPlatform {
    pub fn new(settings: Settings, pg_pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            settings,
            pg_pool,
            redis_client,
            clock: Arc::new(SystemClock),
            capabilities: Arc::new(ApiCapabilityRegistry::new()),
        }
    }
}
