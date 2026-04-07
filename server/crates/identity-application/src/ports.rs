use crate::{AccessTokenClaims, TokenPair};
use crate::{ApplicationError, StoredCredential, UserReadModel};
use async_trait::async_trait;
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, User, UserId};
pub use ordering_food_platform_kernel::Clock;
use ordering_food_shared_kernel::Timestamp;
use std::sync::Arc;

pub trait IdGenerator: Send + Sync {
    fn next_user_id(&self) -> UserId;
}

#[async_trait]
pub trait IdentityUnitOfWork: Send {
    async fn find_user_by_id(&mut self, user_id: &UserId)
    -> Result<Option<User>, ApplicationError>;

    async fn find_user_by_identity(
        &mut self,
        identity_type: &IdentityType,
        identifier: &NormalizedIdentifier,
    ) -> Result<Option<User>, ApplicationError>;

    async fn insert_user(&mut self, user: &User) -> Result<(), ApplicationError>;

    async fn update_user(&mut self, user: &User) -> Result<(), ApplicationError>;

    async fn find_credential_by_user_id(
        &mut self,
        user_id: &UserId,
    ) -> Result<Option<StoredCredential>, ApplicationError>;

    async fn upsert_credential(
        &mut self,
        user_id: &UserId,
        password_hash: &str,
        now: Timestamp,
    ) -> Result<(), ApplicationError>;

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError>;
    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait IdentityUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn IdentityUnitOfWork>, ApplicationError>;
}

#[async_trait]
pub trait UserReadRepository: Send + Sync {
    async fn get_by_id(&self, user_id: &str) -> Result<Option<UserReadModel>, ApplicationError>;
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
        user_id: &str,
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
