use crate::{AccessTokenClaims, TokenPair};
use crate::{ApplicationError, StoredCredential, UserReadModel};
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

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, raw_password: &str) -> Result<String, ApplicationError>;
    async fn verify(&self, raw_password: &str, hash: &str) -> Result<bool, ApplicationError>;
}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn find_by_user_id(
        &self,
        tx: &mut dyn TransactionContext,
        user_id: &UserId,
    ) -> Result<Option<StoredCredential>, ApplicationError>;

    async fn upsert(
        &self,
        tx: &mut dyn TransactionContext,
        user_id: &UserId,
        password_hash: &str,
        now: Timestamp,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait TokenService: Send + Sync {
    fn generate_token_pair(&self, user_id: &str) -> Result<TokenPair, ApplicationError>;
    fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, ApplicationError>;
}

#[async_trait]
pub trait RefreshTokenStore: Send + Sync {
    async fn store(
        &self,
        token: &str,
        user_id: &str,
        ttl_seconds: u64,
    ) -> Result<(), ApplicationError>;

    async fn lookup(&self, token: &str) -> Result<Option<String>, ApplicationError>;
    async fn revoke(&self, token: &str) -> Result<(), ApplicationError>;
    async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), ApplicationError>;
}
