use crate::{ApplicationError, RefreshTokenStore, TokenPair, TokenService, UserReadRepository};
use ordering_food_identity_domain::{UserId, UserStatus};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RefreshTokenInput {
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct RefreshTokenOutput {
    pub token_pair: TokenPair,
    pub user_id: String,
}

pub struct RefreshToken {
    token_service: Arc<dyn TokenService>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
    user_read_repository: Arc<dyn UserReadRepository>,
}

impl RefreshToken {
    pub fn new(
        token_service: Arc<dyn TokenService>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
        user_read_repository: Arc<dyn UserReadRepository>,
    ) -> Self {
        Self {
            token_service,
            refresh_token_store,
            user_read_repository,
        }
    }

    pub async fn execute(
        &self,
        input: RefreshTokenInput,
    ) -> Result<RefreshTokenOutput, ApplicationError> {
        let user_id = self
            .refresh_token_store
            .lookup(&input.refresh_token)
            .await?
            .ok_or_else(|| ApplicationError::unauthorized("invalid refresh token"))?;

        let user = self
            .user_read_repository
            .get_by_id(&UserId::new(&user_id))
            .await?
            .ok_or_else(|| ApplicationError::unauthorized("invalid refresh token"))?;

        let status = UserStatus::parse(&user.status)
            .map_err(|_| ApplicationError::unauthorized("invalid refresh token"))?;
        if status != UserStatus::Active || user.deleted_at.is_some() {
            self.refresh_token_store
                .revoke(&input.refresh_token)
                .await?;
            return Err(ApplicationError::unauthorized("invalid refresh token"));
        }

        // Revoke old token (rotation)
        self.refresh_token_store
            .revoke(&input.refresh_token)
            .await?;

        let token_pair = self.token_service.generate_token_pair(&user_id)?;

        self.refresh_token_store
            .store(
                &token_pair.refresh_token,
                &user_id,
                token_pair.refresh_token_expires_in,
            )
            .await?;

        Ok(RefreshTokenOutput {
            token_pair,
            user_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AccessTokenClaims, ApplicationError, UserIdentityReadModel, UserProfileReadModel,
        UserReadModel,
    };
    use async_trait::async_trait;
    use ordering_food_shared_kernel::{Identifier, Timestamp};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

    #[derive(Default)]
    struct FakeTokenService {
        counter: Mutex<u32>,
    }

    #[async_trait]
    impl TokenService for FakeTokenService {
        fn generate_token_pair(&self, user_id: &str) -> Result<TokenPair, ApplicationError> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            Ok(TokenPair {
                access_token: format!("access-{user_id}-{}", *counter),
                access_token_expires_in: 900,
                refresh_token: format!("refresh-{user_id}-{}", *counter),
                refresh_token_expires_in: 604800,
            })
        }

        fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, ApplicationError> {
            let user_id = token
                .strip_prefix("access-")
                .and_then(|value| value.rsplit_once('-').map(|(prefix, _)| prefix))
                .ok_or_else(|| ApplicationError::unauthorized("invalid or expired access token"))?;
            Ok(AccessTokenClaims {
                user_id: user_id.to_string(),
                exp: 900,
            })
        }
    }

    #[derive(Default)]
    struct FakeRefreshTokenStore {
        tokens: Mutex<HashMap<String, String>>,
    }

    impl FakeRefreshTokenStore {
        fn seed(&self, token: &str, user_id: &str) {
            self.tokens
                .lock()
                .unwrap()
                .insert(token.to_string(), user_id.to_string());
        }
    }

    #[async_trait]
    impl RefreshTokenStore for FakeRefreshTokenStore {
        async fn store(
            &self,
            token: &str,
            user_id: &str,
            _ttl_seconds: u64,
        ) -> Result<(), ApplicationError> {
            self.tokens
                .lock()
                .unwrap()
                .insert(token.to_string(), user_id.to_string());
            Ok(())
        }

        async fn lookup(&self, token: &str) -> Result<Option<String>, ApplicationError> {
            Ok(self.tokens.lock().unwrap().get(token).cloned())
        }

        async fn revoke(&self, token: &str) -> Result<(), ApplicationError> {
            self.tokens.lock().unwrap().remove(token);
            Ok(())
        }

        async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), ApplicationError> {
            self.tokens
                .lock()
                .unwrap()
                .retain(|_, value| value != user_id);
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeUserReadRepository {
        users: Mutex<HashMap<String, UserReadModel>>,
    }

    impl FakeUserReadRepository {
        fn seed(&self, user: UserReadModel) {
            self.users
                .lock()
                .unwrap()
                .insert(user.user_id.clone(), user);
        }
    }

    #[async_trait]
    impl UserReadRepository for FakeUserReadRepository {
        async fn get_by_id(
            &self,
            user_id: &UserId,
        ) -> Result<Option<UserReadModel>, ApplicationError> {
            Ok(self.users.lock().unwrap().get(user_id.as_str()).cloned())
        }
    }

    fn make_user_read_model(
        user_id: &str,
        status: &str,
        deleted_at: Option<Timestamp>,
    ) -> UserReadModel {
        UserReadModel {
            user_id: user_id.to_string(),
            status: status.to_string(),
            profile: UserProfileReadModel {
                display_name: "Alice".to_string(),
                given_name: None,
                family_name: None,
                avatar_url: None,
            },
            identities: vec![UserIdentityReadModel {
                identity_type: "email".to_string(),
                identifier_normalized: "alice@example.com".to_string(),
                bound_at: datetime!(2026-03-10 08:00 UTC),
            }],
            created_at: datetime!(2026-03-10 08:00 UTC),
            updated_at: datetime!(2026-03-10 08:00 UTC),
            deleted_at,
        }
    }

    #[tokio::test]
    async fn refresh_token_rotates_refresh_token() {
        let refresh_token_store = Arc::new(FakeRefreshTokenStore::default());
        refresh_token_store.seed("refresh-old", "user-1");
        let user_read_repository = Arc::new(FakeUserReadRepository::default());
        user_read_repository.seed(make_user_read_model("user-1", "active", None));
        let use_case = RefreshToken::new(
            Arc::new(FakeTokenService::default()),
            refresh_token_store.clone(),
            user_read_repository,
        );

        let output = use_case
            .execute(RefreshTokenInput {
                refresh_token: "refresh-old".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(output.user_id, "user-1");
        assert_eq!(output.token_pair.access_token, "access-user-1-1");
        assert_eq!(output.token_pair.refresh_token, "refresh-user-1-1");
        assert_eq!(
            refresh_token_store
                .tokens
                .lock()
                .unwrap()
                .get("refresh-old"),
            None
        );
        assert_eq!(
            refresh_token_store
                .tokens
                .lock()
                .unwrap()
                .get("refresh-user-1-1")
                .cloned(),
            Some("user-1".to_string())
        );
    }

    #[tokio::test]
    async fn refresh_token_rejects_unknown_token() {
        let use_case = RefreshToken::new(
            Arc::new(FakeTokenService::default()),
            Arc::new(FakeRefreshTokenStore::default()),
            Arc::new(FakeUserReadRepository::default()),
        );

        let error = use_case
            .execute(RefreshTokenInput {
                refresh_token: "missing-token".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Unauthorized { ref message }
            if message == "invalid refresh token"
        ));
    }

    #[tokio::test]
    async fn refresh_token_rejects_disabled_user() {
        let refresh_token_store = Arc::new(FakeRefreshTokenStore::default());
        refresh_token_store.seed("refresh-old", "user-1");
        let user_read_repository = Arc::new(FakeUserReadRepository::default());
        user_read_repository.seed(make_user_read_model("user-1", "disabled", None));
        let use_case = RefreshToken::new(
            Arc::new(FakeTokenService::default()),
            refresh_token_store.clone(),
            user_read_repository,
        );

        let error = use_case
            .execute(RefreshTokenInput {
                refresh_token: "refresh-old".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Unauthorized { ref message }
            if message == "invalid refresh token"
        ));
        assert_eq!(
            refresh_token_store
                .tokens
                .lock()
                .unwrap()
                .get("refresh-old"),
            None
        );
    }
}
