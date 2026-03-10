use crate::{
    config::Settings,
    runtime::{SystemClock, UuidV7UserIdGenerator},
};
use ordering_food_identity_application::{Clock, IdGenerator};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiPlatform {
    pub settings: Settings,
    pub pg_pool: PgPool,
    pub redis_client: redis::Client,
    pub clock: Arc<dyn Clock>,
    pub id_generator: Arc<dyn IdGenerator>,
}

impl ApiPlatform {
    pub fn new(settings: Settings, pg_pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            settings,
            pg_pool,
            redis_client,
            clock: Arc::new(SystemClock),
            id_generator: Arc::new(UuidV7UserIdGenerator),
        }
    }
}
