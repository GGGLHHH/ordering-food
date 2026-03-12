use crate::{
    BindUserIdentity, Clock, CreateUser, CredentialRepository, DisableUser, IdGenerator, Login,
    Logout, PasswordHasher, RefreshToken, RefreshTokenStore, SoftDeleteUser, TokenService,
    TransactionManager, UpdateUserProfile, UserQueryService, UserReadRepository, UserRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct IdentityModule {
    pub create_user: Arc<CreateUser>,
    pub update_user_profile: Arc<UpdateUserProfile>,
    pub bind_user_identity: Arc<BindUserIdentity>,
    pub disable_user: Arc<DisableUser>,
    pub soft_delete_user: Arc<SoftDeleteUser>,
    pub user_queries: Arc<UserQueryService>,
    pub login: Arc<Login>,
    pub refresh_token: Arc<RefreshToken>,
    pub logout: Arc<Logout>,
}

impl IdentityModule {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        user_read_repository: Arc<dyn UserReadRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        credential_repository: Arc<dyn CredentialRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        token_service: Arc<dyn TokenService>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
    ) -> Self {
        Self {
            create_user: Arc::new(CreateUser::new(
                user_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
                id_generator,
                password_hasher.clone(),
                credential_repository.clone(),
            )),
            update_user_profile: Arc::new(UpdateUserProfile::new(
                user_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            bind_user_identity: Arc::new(BindUserIdentity::new(
                user_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            disable_user: Arc::new(DisableUser::new(
                user_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            soft_delete_user: Arc::new(SoftDeleteUser::new(
                user_repository.clone(),
                transaction_manager.clone(),
                clock,
            )),
            user_queries: Arc::new(UserQueryService::new(user_read_repository.clone())),
            login: Arc::new(Login::new(
                user_repository,
                credential_repository,
                transaction_manager,
                password_hasher,
                token_service.clone(),
                refresh_token_store.clone(),
            )),
            refresh_token: Arc::new(RefreshToken::new(
                token_service,
                refresh_token_store.clone(),
                user_read_repository.clone(),
            )),
            logout: Arc::new(Logout::new(refresh_token_store)),
        }
    }
}
