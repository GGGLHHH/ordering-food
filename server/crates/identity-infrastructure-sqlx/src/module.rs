use crate::{SqlxTransactionManager, SqlxUserReadRepository, SqlxUserRepository};
use ordering_food_identity_application::{Clock, IdGenerator, IdentityModule};
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_identity_module(
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
) -> Arc<IdentityModule> {
    let repository = Arc::new(SqlxUserRepository);
    let read_repository = Arc::new(SqlxUserReadRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));

    Arc::new(IdentityModule::new(
        repository,
        read_repository,
        transaction_manager,
        clock,
        id_generator,
    ))
}
