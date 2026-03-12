use crate::{
    SqlxCredentialRepository, SqlxTransactionManager, SqlxUserReadRepository, SqlxUserRepository,
};
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
    let repository = Arc::new(SqlxUserRepository);
    let read_repository = Arc::new(SqlxUserReadRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));
    let credential_repository = Arc::new(SqlxCredentialRepository);

    Arc::new(IdentityModule::new(
        repository,
        read_repository,
        transaction_manager,
        clock,
        id_generator,
        credential_repository,
        password_hasher,
        token_service,
        refresh_token_store,
    ))
}
