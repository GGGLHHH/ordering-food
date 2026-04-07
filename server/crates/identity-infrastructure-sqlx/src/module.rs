use crate::{SqlxIdentityUnitOfWorkFactory, SqlxUserReadRepository};
use ordering_food_identity_application::{
    Clock, IdGenerator, IdentityModule, PasswordHasher, RefreshTokenStore, TokenService,
};
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_identity_module(
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    password_hasher: Arc<dyn PasswordHasher>,
    token_service: Arc<dyn TokenService>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
) -> Arc<IdentityModule> {
    let read_repository = Arc::new(SqlxUserReadRepository::new(pool.clone()));
    let unit_of_work_factory = Arc::new(SqlxIdentityUnitOfWorkFactory::new(pool));

    Arc::new(IdentityModule::new(
        read_repository,
        unit_of_work_factory,
        clock,
        id_generator,
        password_hasher,
        token_service,
        refresh_token_store,
    ))
}
