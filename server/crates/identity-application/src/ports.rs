use crate::{ApplicationError, UserReadModel};
use async_trait::async_trait;
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, User, UserId};
use ordering_food_shared_kernel::Timestamp;
use std::any::Any;
use std::sync::Arc;

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

pub trait IdGenerator: Send + Sync {
    fn next_user_id(&self) -> UserId;
}

pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError>;
    async fn commit(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
    async fn rollback(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        user_id: &UserId,
    ) -> Result<Option<User>, ApplicationError>;

    async fn find_by_identity(
        &self,
        tx: &mut dyn TransactionContext,
        identity_type: &IdentityType,
        identifier: &NormalizedIdentifier,
    ) -> Result<Option<User>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        user: &User,
    ) -> Result<(), ApplicationError>;

    async fn update(
        &self,
        tx: &mut dyn TransactionContext,
        user: &User,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait UserReadRepository: Send + Sync {
    async fn get_by_id(&self, user_id: &UserId) -> Result<Option<UserReadModel>, ApplicationError>;
}

#[derive(Clone)]
pub struct UserQueryService {
    repository: Arc<dyn UserReadRepository>,
}

impl UserQueryService {
    pub fn new(repository: Arc<dyn UserReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_by_id(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserReadModel>, ApplicationError> {
        self.repository.get_by_id(user_id).await
    }
}
