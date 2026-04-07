use crate::{
    BindUserIdentity, Clock, CreateUser, DisableUser, IdGenerator, IdentityUnitOfWorkFactory,
    Login, Logout, PasswordHasher, RefreshToken, RefreshTokenStore, SoftDeleteUser, TokenService,
    UpdateUserProfile, UserQueryService, UserReadRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct IdentityModule {
    create_user: Arc<CreateUser>,
    update_user_profile: Arc<UpdateUserProfile>,
    bind_user_identity: Arc<BindUserIdentity>,
    disable_user: Arc<DisableUser>,
    soft_delete_user: Arc<SoftDeleteUser>,
    user_queries: Arc<UserQueryService>,
    login: Arc<Login>,
    refresh_token: Arc<RefreshToken>,
    logout: Arc<Logout>,
}

impl IdentityModule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_read_repository: Arc<dyn UserReadRepository>,
        unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        password_hasher: Arc<dyn PasswordHasher>,
        token_service: Arc<dyn TokenService>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
    ) -> Self {
        Self {
            create_user: Arc::new(CreateUser::new(
                unit_of_work_factory.clone(),
                clock.clone(),
                id_generator,
                password_hasher.clone(),
            )),
            update_user_profile: Arc::new(UpdateUserProfile::new(
                unit_of_work_factory.clone(),
                clock.clone(),
            )),
            bind_user_identity: Arc::new(BindUserIdentity::new(
                unit_of_work_factory.clone(),
                clock.clone(),
            )),
            disable_user: Arc::new(DisableUser::new(
                unit_of_work_factory.clone(),
                clock.clone(),
            )),
            soft_delete_user: Arc::new(SoftDeleteUser::new(unit_of_work_factory.clone(), clock)),
            user_queries: Arc::new(UserQueryService::new(user_read_repository.clone())),
            login: Arc::new(Login::new(
                unit_of_work_factory,
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

    pub fn create_user(&self) -> &Arc<CreateUser> {
        &self.create_user
    }

    pub fn update_user_profile(&self) -> &Arc<UpdateUserProfile> {
        &self.update_user_profile
    }

    pub fn bind_user_identity(&self) -> &Arc<BindUserIdentity> {
        &self.bind_user_identity
    }

    pub fn disable_user(&self) -> &Arc<DisableUser> {
        &self.disable_user
    }

    pub fn soft_delete_user(&self) -> &Arc<SoftDeleteUser> {
        &self.soft_delete_user
    }

    pub fn user_queries(&self) -> &Arc<UserQueryService> {
        &self.user_queries
    }

    pub fn login(&self) -> &Arc<Login> {
        &self.login
    }

    pub fn refresh_token(&self) -> &Arc<RefreshToken> {
        &self.refresh_token
    }

    pub fn logout(&self) -> &Arc<Logout> {
        &self.logout
    }
}
