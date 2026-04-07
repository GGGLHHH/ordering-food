use crate::{
    ApplicationError, Clock, IdGenerator, IdentityUnitOfWorkFactory, PasswordHasher,
    UserIdentityReadModel, UserProfileReadModel, UserReadModel,
};
use ordering_food_identity_domain::{
    IdentityType, NormalizedIdentifier, User, UserIdentity, UserProfile,
};
use ordering_food_shared_kernel::Identifier;
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
    pub password: Option<String>,
}

pub struct CreateUser {
    unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    password_hasher: Arc<dyn PasswordHasher>,
}

impl CreateUser {
    pub fn new(
        unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        password_hasher: Arc<dyn PasswordHasher>,
    ) -> Self {
        Self {
            unit_of_work_factory,
            clock,
            id_generator,
            password_hasher,
        }
    }

    pub async fn execute(&self, input: CreateUserInput) -> Result<UserReadModel, ApplicationError> {
        let now = self.clock.now();
        let profile = UserProfile::new(
            input.display_name,
            input.given_name,
            input.family_name,
            input.avatar_url,
        )?;
        let identities = input
            .identities
            .into_iter()
            .map(|identity| -> Result<UserIdentity, ApplicationError> {
                let identity_type = IdentityType::parse(identity.identity_type)?;
                let identifier = NormalizedIdentifier::new(identity.identifier)?;
                Ok(UserIdentity::new(identity_type, identifier, now))
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        let password_hash = match input.password {
            Some(raw_password) => Some(self.password_hasher.hash(&raw_password).await?),
            None => None,
        };
        let mut user = User::create(self.id_generator.next_user_id(), profile, now);
        for identity in identities {
            user.bind_identity(identity, now)?;
        }

        let mut unit_of_work = self.unit_of_work_factory.begin().await?;

        for identity in user.identities() {
            match unit_of_work
                .find_user_by_identity(identity.identity_type(), identity.identifier_normalized())
                .await
            {
                Ok(Some(_)) => {
                    unit_of_work.rollback().await?;
                    return Err(ApplicationError::conflict(
                        "identity is already bound to another user",
                    ));
                }
                Ok(None) => {}
                Err(error) => {
                    unit_of_work.rollback().await?;
                    return Err(error);
                }
            }
        }

        if let Err(error) = unit_of_work.insert_user(&user).await {
            unit_of_work.rollback().await?;
            return Err(error);
        }

        if let Some(password_hash) = password_hash.as_deref()
            && let Err(error) = unit_of_work
                .upsert_credential(user.id(), password_hash, now)
                .await
        {
            unit_of_work.rollback().await?;
            return Err(error);
        }

        unit_of_work.commit().await?;
        Ok(map_user_to_read_model(&user))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApplicationError, IdGenerator, PasswordHasher,
        testing::{FakeClock, FakeIdentityStore, FakeIdentityUnitOfWorkFactory},
    };
    use async_trait::async_trait;
    use ordering_food_identity_domain::{NormalizedIdentifier, UserId};
    use ordering_food_shared_kernel::Timestamp;
    use std::sync::Arc;
    use time::macros::datetime;

    struct FakeIdGenerator {
        next_id: UserId,
    }

    impl IdGenerator for FakeIdGenerator {
        fn next_user_id(&self) -> UserId {
            self.next_id.clone()
        }
    }

    #[derive(Default)]
    struct FakePasswordHasher;

    #[async_trait]
    impl PasswordHasher for FakePasswordHasher {
        async fn hash(&self, raw_password: &str) -> Result<String, ApplicationError> {
            Ok(format!("hashed:{raw_password}"))
        }

        async fn verify(&self, _raw_password: &str, _hash: &str) -> Result<bool, ApplicationError> {
            Ok(true)
        }
    }

    fn build_create_user(
        transactions: Arc<FakeIdentityUnitOfWorkFactory>,
        now: Timestamp,
        next_id: UserId,
    ) -> CreateUser {
        CreateUser::new(
            transactions,
            Arc::new(FakeClock { now }),
            Arc::new(FakeIdGenerator { next_id }),
            Arc::new(FakePasswordHasher),
        )
    }

    #[tokio::test]
    async fn create_user_generates_id_and_persists_aggregate() {
        let store = Arc::new(FakeIdentityStore::default());
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let create_user = build_create_user(
            transactions.clone(),
            datetime!(2026-03-10 08:00 UTC),
            UserId::new("user-1"),
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
                password: None,
            })
            .await
            .unwrap();

        assert_eq!(user.user_id, "user-1");
        assert_eq!(user.identities.len(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
        assert!(store.users().contains_key("user-1"));
        assert!(store.credentials().is_empty());
    }

    #[tokio::test]
    async fn create_user_rolls_back_when_identity_conflicts() {
        let store = Arc::new(FakeIdentityStore::default());
        store.seed_user(
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

        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store));
        let create_user = build_create_user(
            transactions.clone(),
            datetime!(2026-03-10 08:00 UTC),
            UserId::new("user-2"),
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
                password: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Conflict { .. }));
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn create_user_persists_hashed_password_when_password_is_present() {
        let store = Arc::new(FakeIdentityStore::default());
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let create_user = build_create_user(
            transactions.clone(),
            datetime!(2026-03-10 08:00 UTC),
            UserId::new("user-3"),
        );

        let user = create_user
            .execute(CreateUserInput {
                display_name: "Alice".to_string(),
                given_name: None,
                family_name: None,
                avatar_url: None,
                identities: vec![CreateUserIdentityInput {
                    identity_type: "email".to_string(),
                    identifier: "Alice@Example.com".to_string(),
                }],
                password: Some("secret123".to_string()),
            })
            .await
            .unwrap();

        let credentials = store.credentials();
        let credential = credentials.get(&user.user_id).unwrap();
        assert_eq!(credential.user_id, user.user_id);
        assert_eq!(credential.password_hash, "hashed:secret123");
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn create_user_skips_credential_write_when_password_is_absent() {
        let store = Arc::new(FakeIdentityStore::default());
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let create_user = build_create_user(
            transactions,
            datetime!(2026-03-10 08:00 UTC),
            UserId::new("user-4"),
        );

        create_user
            .execute(CreateUserInput {
                display_name: "Alice".to_string(),
                given_name: None,
                family_name: None,
                avatar_url: None,
                identities: vec![CreateUserIdentityInput {
                    identity_type: "email".to_string(),
                    identifier: "Alice@Example.com".to_string(),
                }],
                password: None,
            })
            .await
            .unwrap();

        assert!(store.credentials().is_empty());
    }

    #[tokio::test]
    async fn create_user_rolls_back_when_credential_upsert_fails() {
        let store = Arc::new(FakeIdentityStore::default());
        store.fail_on_credential_upsert();
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let create_user = build_create_user(
            transactions.clone(),
            datetime!(2026-03-10 08:00 UTC),
            UserId::new("user-5"),
        );

        let error = create_user
            .execute(CreateUserInput {
                display_name: "Alice".to_string(),
                given_name: None,
                family_name: None,
                avatar_url: None,
                identities: vec![CreateUserIdentityInput {
                    identity_type: "email".to_string(),
                    identifier: "Alice@Example.com".to_string(),
                }],
                password: Some("secret123".to_string()),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 0);
        assert!(store.credentials().is_empty());
        assert!(!store.users().contains_key("user-5"));
    }
}
