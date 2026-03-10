use crate::{ApplicationError, Clock, IdGenerator, TransactionManager, UserRepository};
use ordering_food_identity_domain::{
    IdentityType, NormalizedIdentifier, User, UserIdentity, UserProfile,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateUserIdentityInput {
    pub identity_type: String,
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateUserInput {
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub avatar_url: Option<String>,
    pub identities: Vec<CreateUserIdentityInput>,
}

pub struct CreateUser {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateUser {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateUserInput) -> Result<User, ApplicationError> {
        let now = self.clock.now();
        let profile = UserProfile::new(
            input.display_name,
            input.given_name,
            input.family_name,
            input.avatar_url,
        )?;
        let mut user = User::create(self.id_generator.next_user_id(), profile, now);
        let mut tx = self.transaction_manager.begin().await?;

        for identity in input.identities {
            let identity_type = IdentityType::parse(identity.identity_type)?;
            let identifier = NormalizedIdentifier::new(identity.identifier)?;

            if self
                .repository
                .find_by_identity(tx.as_mut(), &identity_type, &identifier)
                .await?
                .is_some()
            {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::conflict(
                    "identity is already bound to another user",
                ));
            }

            if let Err(error) =
                user.bind_identity(UserIdentity::new(identity_type, identifier, now), now)
            {
                self.transaction_manager.rollback(tx).await?;
                return Err(error.into());
            }
        }

        if let Err(error) = self.repository.insert(tx.as_mut(), &user).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApplicationError, Clock, IdGenerator, TransactionContext, TransactionManager,
        UserReadRepository, UserRepository,
    };
    use async_trait::async_trait;
    use ordering_food_identity_domain::{NormalizedIdentifier, UserId};
    use ordering_food_shared_kernel::{Identifier, Timestamp};
    use std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

    #[derive(Default)]
    struct FakeTransactionContext;

    impl TransactionContext for FakeTransactionContext {
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
            self
        }
    }

    #[derive(Default)]
    struct FakeTransactionManager {
        begin_count: Mutex<u32>,
        commit_count: Mutex<u32>,
        rollback_count: Mutex<u32>,
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

    struct FakeClock {
        now: Timestamp,
    }

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FakeIdGenerator {
        next_id: UserId,
    }

    impl IdGenerator for FakeIdGenerator {
        fn next_user_id(&self) -> UserId {
            self.next_id.clone()
        }
    }

    #[derive(Default)]
    struct FakeRepository {
        users: Mutex<HashMap<String, User>>,
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
        ) -> Result<Option<crate::UserReadModel>, ApplicationError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn create_user_generates_id_and_persists_aggregate() {
        let repository = Arc::new(FakeRepository::default());
        let transactions = Arc::new(FakeTransactionManager::default());
        let create_user = CreateUser::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 08:00 UTC),
            }),
            Arc::new(FakeIdGenerator {
                next_id: UserId::new("user-1"),
            }),
        );

        let user = create_user
            .execute(CreateUserInput {
                display_name: "Alice".to_string(),
                given_name: Some("Alice".to_string()),
                family_name: None,
                avatar_url: None,
                identities: vec![CreateUserIdentityInput {
                    identity_type: "email".to_string(),
                    identifier: "Alice@Example.com".to_string(),
                }],
            })
            .await
            .unwrap();

        assert_eq!(user.id().as_str(), "user-1");
        assert_eq!(user.identities().len(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
        assert!(repository.users.lock().unwrap().contains_key("user-1"));
    }

    #[tokio::test]
    async fn create_user_rolls_back_when_identity_conflicts() {
        let repository = Arc::new(FakeRepository::default());
        repository.users.lock().unwrap().insert(
            "existing".to_string(),
            User::rehydrate(
                UserId::new("existing"),
                ordering_food_identity_domain::UserStatus::Active,
                UserProfile::new("Existing", None, None, None).unwrap(),
                vec![UserIdentity::new(
                    IdentityType::Email,
                    NormalizedIdentifier::new("existing@example.com").unwrap(),
                    datetime!(2026-03-10 07:00 UTC),
                )],
                datetime!(2026-03-10 07:00 UTC),
                datetime!(2026-03-10 07:00 UTC),
                None,
            )
            .unwrap(),
        );

        let transactions = Arc::new(FakeTransactionManager::default());
        let create_user = CreateUser::new(
            repository,
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 08:00 UTC),
            }),
            Arc::new(FakeIdGenerator {
                next_id: UserId::new("user-2"),
            }),
        );

        let error = create_user
            .execute(CreateUserInput {
                display_name: "Bob".to_string(),
                given_name: None,
                family_name: None,
                avatar_url: None,
                identities: vec![CreateUserIdentityInput {
                    identity_type: "email".to_string(),
                    identifier: "existing@example.com".to_string(),
                }],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Conflict { .. }));
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 1);
    }
}
