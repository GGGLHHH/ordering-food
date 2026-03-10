use crate::{
    ApplicationError, Clock, TransactionContext, TransactionManager, UserReadModel,
    UserReadRepository, UserRepository,
};
use async_trait::async_trait;
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, User, UserId};
use ordering_food_shared_kernel::{Identifier, Timestamp};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

#[derive(Default)]
pub struct FakeTransactionContext;

impl TransactionContext for FakeTransactionContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

#[derive(Default)]
pub struct FakeTransactionManager {
    pub begin_count: Mutex<u32>,
    pub commit_count: Mutex<u32>,
    pub rollback_count: Mutex<u32>,
}

#[async_trait]
impl TransactionManager for FakeTransactionManager {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
        *self.begin_count.lock().unwrap() += 1;
        Ok(Box::new(FakeTransactionContext))
    }

    async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        *self.commit_count.lock().unwrap() += 1;
        Ok(())
    }

    async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        *self.rollback_count.lock().unwrap() += 1;
        Ok(())
    }
}

pub struct FakeClock {
    pub now: Timestamp,
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

#[derive(Default)]
pub struct FakeRepository {
    users: Mutex<HashMap<String, User>>,
}

impl FakeRepository {
    pub fn seed(&self, user: User) {
        self.users
            .lock()
            .unwrap()
            .insert(user.id().as_str().to_string(), user);
    }

    pub fn users(&self) -> MutexGuard<'_, HashMap<String, User>> {
        self.users.lock().unwrap()
    }
}

#[async_trait]
impl UserRepository for FakeRepository {
    async fn find_by_id(
        &self,
        _tx: &mut dyn TransactionContext,
        user_id: &UserId,
    ) -> Result<Option<User>, ApplicationError> {
        Ok(self.users.lock().unwrap().get(user_id.as_str()).cloned())
    }

    async fn find_by_identity(
        &self,
        _tx: &mut dyn TransactionContext,
        identity_type: &IdentityType,
        identifier: &NormalizedIdentifier,
    ) -> Result<Option<User>, ApplicationError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .values()
            .find(|user| {
                user.identities().iter().any(|identity| {
                    identity.identity_type() == identity_type
                        && identity.identifier_normalized() == identifier
                })
            })
            .cloned())
    }

    async fn insert(
        &self,
        _tx: &mut dyn TransactionContext,
        user: &User,
    ) -> Result<(), ApplicationError> {
        self.users
            .lock()
            .unwrap()
            .insert(user.id().as_str().to_string(), user.clone());
        Ok(())
    }

    async fn update(
        &self,
        _tx: &mut dyn TransactionContext,
        user: &User,
    ) -> Result<(), ApplicationError> {
        self.users
            .lock()
            .unwrap()
            .insert(user.id().as_str().to_string(), user.clone());
        Ok(())
    }
}

#[async_trait]
impl UserReadRepository for FakeRepository {
    async fn get_by_id(
        &self,
        _user_id: &UserId,
    ) -> Result<Option<UserReadModel>, ApplicationError> {
        Ok(None)
    }
}
