use crate::{
    ApplicationError, CredentialRepository, PasswordHasher, RefreshTokenStore, TokenPair,
    TokenService, TransactionManager, UserRepository,
};
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, UserStatus};
use ordering_food_shared_kernel::Identifier;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LoginInput {
    pub identity_type: String,
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct LoginOutput {
    pub token_pair: TokenPair,
    pub user_id: String,
}

pub struct Login {
    user_repository: Arc<dyn UserRepository>,
    credential_repository: Arc<dyn CredentialRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    password_hasher: Arc<dyn PasswordHasher>,
    token_service: Arc<dyn TokenService>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
}

impl Login {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        credential_repository: Arc<dyn CredentialRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        password_hasher: Arc<dyn PasswordHasher>,
        token_service: Arc<dyn TokenService>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
    ) -> Self {
        Self {
            user_repository,
            credential_repository,
            transaction_manager,
            password_hasher,
            token_service,
            refresh_token_store,
        }
    }

    pub async fn execute(&self, input: LoginInput) -> Result<LoginOutput, ApplicationError> {
        let identity_type = IdentityType::parse(&input.identity_type)?;
        let identifier = NormalizedIdentifier::new(&input.identifier)?;

        let mut tx = self.transaction_manager.begin().await?;

        let user = self
            .user_repository
            .find_by_identity(tx.as_mut(), &identity_type, &identifier)
            .await?;

        let user = match user {
            Some(u) => u,
            None => {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::unauthorized("invalid credentials"));
            }
        };

        if user.is_deleted() || user.status() != UserStatus::Active {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::unauthorized("invalid credentials"));
        }

        let user_id_str = Identifier::as_str(user.id()).to_string();

        let credential = self
            .credential_repository
            .find_by_user_id(tx.as_mut(), user.id())
            .await?;

        self.transaction_manager.commit(tx).await?;

        let credential = match credential {
            Some(c) => c,
            None => return Err(ApplicationError::unauthorized("invalid credentials")),
        };

        let valid = self
            .password_hasher
            .verify(&input.password, &credential.password_hash)
            .await?;

        if !valid {
            return Err(ApplicationError::unauthorized("invalid credentials"));
        }

        let token_pair = self.token_service.generate_token_pair(&user_id_str)?;

        self.refresh_token_store
            .store(
                &token_pair.refresh_token,
                &user_id_str,
                token_pair.refresh_token_expires_in,
            )
            .await?;

        Ok(LoginOutput {
            token_pair,
            user_id: user_id_str,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AccessTokenClaims, ApplicationError, CredentialRepository, PasswordHasher,
        RefreshTokenStore, StoredCredential, TokenService,
        testing::{FakeRepository, FakeTransactionManager},
    };
    use async_trait::async_trait;
    use ordering_food_identity_domain::{
        IdentityType, NormalizedIdentifier, User, UserId, UserIdentity, UserProfile, UserStatus,
    };
    use ordering_food_shared_kernel::Timestamp;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

    #[derive(Default)]
    struct FakeCredentialRepository {
        credentials: Mutex<HashMap<String, StoredCredential>>,
    }

    impl FakeCredentialRepository {
        fn seed(&self, credential: StoredCredential) {
            self.credentials
                .lock()
                .unwrap()
                .insert(credential.user_id.clone(), credential);
        }
    }

    #[async_trait]
    impl CredentialRepository for FakeCredentialRepository {
        async fn find_by_user_id(
            &self,
            _tx: &mut dyn crate::TransactionContext,
            user_id: &UserId,
        ) -> Result<Option<StoredCredential>, ApplicationError> {
            Ok(self
                .credentials
                .lock()
                .unwrap()
                .get(user_id.as_str())
                .cloned())
        }

        async fn upsert(
            &self,
            _tx: &mut dyn crate::TransactionContext,
            user_id: &UserId,
            password_hash: &str,
            now: Timestamp,
        ) -> Result<(), ApplicationError> {
            self.credentials.lock().unwrap().insert(
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
    }

    struct FakePasswordHasher;

    #[async_trait]
    impl PasswordHasher for FakePasswordHasher {
        async fn hash(&self, raw_password: &str) -> Result<String, ApplicationError> {
            Ok(format!("hashed:{raw_password}"))
        }

        async fn verify(&self, raw_password: &str, hash: &str) -> Result<bool, ApplicationError> {
            Ok(hash == format!("hashed:{raw_password}"))
        }
    }

    struct FakeTokenService;

    #[async_trait]
    impl TokenService for FakeTokenService {
        fn generate_token_pair(&self, user_id: &str) -> Result<TokenPair, ApplicationError> {
            Ok(TokenPair {
                access_token: format!("access-{user_id}"),
                access_token_expires_in: 900,
                refresh_token: format!("refresh-{user_id}"),
                refresh_token_expires_in: 604800,
            })
        }

        fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, ApplicationError> {
            let user_id = token
                .strip_prefix("access-")
                .ok_or_else(|| ApplicationError::unauthorized("invalid or expired access token"))?;
            Ok(AccessTokenClaims {
                user_id: user_id.to_string(),
                exp: 900,
            })
        }
    }

    #[derive(Default)]
    struct FakeRefreshTokenStore {
        stored: Mutex<HashMap<String, String>>,
    }

    #[async_trait]
    impl RefreshTokenStore for FakeRefreshTokenStore {
        async fn store(
            &self,
            token: &str,
            user_id: &str,
            _ttl_seconds: u64,
        ) -> Result<(), ApplicationError> {
            self.stored
                .lock()
                .unwrap()
                .insert(token.to_string(), user_id.to_string());
            Ok(())
        }

        async fn lookup(&self, token: &str) -> Result<Option<String>, ApplicationError> {
            Ok(self.stored.lock().unwrap().get(token).cloned())
        }

        async fn revoke(&self, token: &str) -> Result<(), ApplicationError> {
            self.stored.lock().unwrap().remove(token);
            Ok(())
        }

        async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), ApplicationError> {
            self.stored
                .lock()
                .unwrap()
                .retain(|_, value| value != user_id);
            Ok(())
        }
    }

    fn make_user(user_id: &str, email: &str, status: UserStatus) -> User {
        let created_at = datetime!(2026-03-10 08:00 UTC);
        let mut user = User::create(
            UserId::new(user_id),
            UserProfile::new("Alice", None, None, None).unwrap(),
            created_at,
        );
        user.bind_identity(
            UserIdentity::new(
                IdentityType::Email,
                NormalizedIdentifier::new(email).unwrap(),
                created_at,
            ),
            created_at,
        )
        .unwrap();

        if status == UserStatus::Disabled {
            user.disable(datetime!(2026-03-10 09:00 UTC)).unwrap();
        }

        user
    }

    fn build_login_use_case(
        repository: Arc<FakeRepository>,
        credential_repository: Arc<FakeCredentialRepository>,
        refresh_token_store: Arc<FakeRefreshTokenStore>,
        transactions: Arc<FakeTransactionManager>,
    ) -> Login {
        Login::new(
            repository,
            credential_repository,
            transactions,
            Arc::new(FakePasswordHasher),
            Arc::new(FakeTokenService),
            refresh_token_store,
        )
    }

    #[tokio::test]
    async fn login_returns_token_pair_for_active_user() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user("user-1", "alice@example.com", UserStatus::Active));

        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 08:00 UTC),
            updated_at: datetime!(2026-03-10 08:00 UTC),
        });

        let refresh_token_store = Arc::new(FakeRefreshTokenStore::default());
        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = build_login_use_case(
            repository,
            credential_repository,
            refresh_token_store.clone(),
            transactions.clone(),
        );

        let output = use_case
            .execute(LoginInput {
                identity_type: "email".to_string(),
                identifier: "Alice@Example.com".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(output.user_id, "user-1");
        assert_eq!(output.token_pair.access_token, "access-user-1");
        assert_eq!(output.token_pair.refresh_token, "refresh-user-1");
        assert_eq!(
            refresh_token_store
                .stored
                .lock()
                .unwrap()
                .get("refresh-user-1")
                .cloned(),
            Some("user-1".to_string())
        );
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn login_rejects_disabled_user() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user(
            "user-1",
            "alice@example.com",
            UserStatus::Disabled,
        ));

        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 08:00 UTC),
            updated_at: datetime!(2026-03-10 08:00 UTC),
        });

        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = build_login_use_case(
            repository,
            credential_repository,
            Arc::new(FakeRefreshTokenStore::default()),
            transactions.clone(),
        );

        let error = use_case
            .execute(LoginInput {
                identity_type: "email".to_string(),
                identifier: "alice@example.com".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Unauthorized { ref message }
            if message == "invalid credentials"
        ));
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn login_rejects_invalid_password() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user("user-1", "alice@example.com", UserStatus::Active));

        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 08:00 UTC),
            updated_at: datetime!(2026-03-10 08:00 UTC),
        });

        let use_case = build_login_use_case(
            repository,
            credential_repository,
            Arc::new(FakeRefreshTokenStore::default()),
            Arc::new(FakeTransactionManager::default()),
        );

        let error = use_case
            .execute(LoginInput {
                identity_type: "email".to_string(),
                identifier: "alice@example.com".to_string(),
                password: "wrong-password".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Unauthorized { ref message }
            if message == "invalid credentials"
        ));
    }
}
