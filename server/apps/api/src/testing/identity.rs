use async_trait::async_trait;
use ordering_food_identity_application::{
    ApplicationError, IdentityUnitOfWork, IdentityUnitOfWorkFactory, StoredCredential,
    UserIdentityReadModel, UserProfileReadModel, UserReadModel, UserReadRepository,
};
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, User, UserId};
use ordering_food_shared_kernel::Identifier;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct FakeIdentityStore {
    users: Mutex<HashMap<String, User>>,
    credentials: Mutex<HashMap<String, StoredCredential>>,
}

impl FakeIdentityStore {
    pub fn seed_user(&self, user: User) {
        self.users
            .lock()
            .unwrap()
            .insert(user.id().as_str().to_string(), user);
    }

    pub fn seed_credential(&self, credential: StoredCredential) {
        self.credentials
            .lock()
            .unwrap()
            .insert(credential.user_id.clone(), credential);
    }
}

#[async_trait]
impl UserReadRepository for FakeIdentityStore {
    async fn get_by_id(&self, user_id: &UserId) -> Result<Option<UserReadModel>, ApplicationError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .get(user_id.as_str())
            .map(map_user_to_read_model))
    }
}

pub struct FakeIdentityUnitOfWorkFactory {
    store: Arc<FakeIdentityStore>,
}

impl FakeIdentityUnitOfWorkFactory {
    pub fn new(store: Arc<FakeIdentityStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl IdentityUnitOfWorkFactory for FakeIdentityUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn IdentityUnitOfWork>, ApplicationError> {
        let users = self.store.users.lock().unwrap().clone();
        let credentials = self.store.credentials.lock().unwrap().clone();

        Ok(Box::new(FakeIdentityUnitOfWork {
            store: self.store.clone(),
            users,
            credentials,
        }))
    }
}

struct FakeIdentityUnitOfWork {
    store: Arc<FakeIdentityStore>,
    users: HashMap<String, User>,
    credentials: HashMap<String, StoredCredential>,
}

#[async_trait]
impl IdentityUnitOfWork for FakeIdentityUnitOfWork {
    async fn find_user_by_id(
        &mut self,
        user_id: &UserId,
    ) -> Result<Option<User>, ApplicationError> {
        Ok(self.users.get(user_id.as_str()).cloned())
    }

    async fn find_user_by_identity(
        &mut self,
        identity_type: &IdentityType,
        identifier: &NormalizedIdentifier,
    ) -> Result<Option<User>, ApplicationError> {
        Ok(self
            .users
            .values()
            .find(|user| {
                user.identities().iter().any(|identity| {
                    identity.identity_type() == identity_type
                        && identity.identifier_normalized() == identifier
                })
            })
            .cloned())
    }

    async fn insert_user(&mut self, user: &User) -> Result<(), ApplicationError> {
        self.users
            .insert(user.id().as_str().to_string(), user.clone());
        Ok(())
    }

    async fn update_user(&mut self, user: &User) -> Result<(), ApplicationError> {
        if !self.users.contains_key(user.id().as_str()) {
            return Err(ApplicationError::not_found("user was not found"));
        }

        self.users
            .insert(user.id().as_str().to_string(), user.clone());
        Ok(())
    }

    async fn find_credential_by_user_id(
        &mut self,
        user_id: &UserId,
    ) -> Result<Option<StoredCredential>, ApplicationError> {
        Ok(self.credentials.get(user_id.as_str()).cloned())
    }

    async fn upsert_credential(
        &mut self,
        user_id: &UserId,
        password_hash: &str,
        now: ordering_food_shared_kernel::Timestamp,
    ) -> Result<(), ApplicationError> {
        self.credentials.insert(
            user_id.as_str().to_string(),
            StoredCredential {
                user_id: user_id.as_str().to_string(),
                password_hash: password_hash.to_string(),
                created_at: now,
                updated_at: now,
            },
        );
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self {
            store,
            users,
            credentials,
        } = *self;

        *store.users.lock().unwrap() = users;
        *store.credentials.lock().unwrap() = credentials;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        Ok(())
    }
}

fn map_user_to_read_model(user: &User) -> UserReadModel {
    UserReadModel {
        user_id: user.id().as_str().to_string(),
        status: user.status().as_str().to_string(),
        profile: UserProfileReadModel {
            display_name: user.profile().display_name().to_string(),
            given_name: user.profile().given_name().map(ToOwned::to_owned),
            family_name: user.profile().family_name().map(ToOwned::to_owned),
            avatar_url: user.profile().avatar_url().map(ToOwned::to_owned),
        },
        identities: user
            .identities()
            .iter()
            .map(|identity| UserIdentityReadModel {
                identity_type: identity.identity_type().as_str().to_string(),
                identifier_normalized: identity.identifier_normalized().as_str().to_string(),
                bound_at: identity.bound_at(),
            })
            .collect(),
        created_at: user.created_at(),
        updated_at: user.updated_at(),
        deleted_at: user.deleted_at(),
    }
}
