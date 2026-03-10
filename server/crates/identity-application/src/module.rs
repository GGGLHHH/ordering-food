use crate::{
    BindUserIdentity, Clock, CreateUser, DisableUser, IdGenerator, SoftDeleteUser,
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
}

impl IdentityModule {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        user_read_repository: Arc<dyn UserReadRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            create_user: Arc::new(CreateUser::new(
                user_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
                id_generator,
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
                user_repository,
                transaction_manager,
                clock,
            )),
            user_queries: Arc::new(UserQueryService::new(user_read_repository)),
        }
    }
}
