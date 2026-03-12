use crate::{ApplicationError, RefreshTokenStore};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LogoutInput {
    pub refresh_token: String,
}

pub struct Logout {
    refresh_token_store: Arc<dyn RefreshTokenStore>,
}

impl Logout {
    pub fn new(refresh_token_store: Arc<dyn RefreshTokenStore>) -> Self {
        Self {
            refresh_token_store,
        }
    }

    pub async fn execute(&self, input: LogoutInput) -> Result<(), ApplicationError> {
        self.refresh_token_store.revoke(&input.refresh_token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeRefreshTokenStore {
        revoked_tokens: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl RefreshTokenStore for FakeRefreshTokenStore {
        async fn store(
            &self,
            _token: &str,
            _user_id: &str,
            _ttl_seconds: u64,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn lookup(&self, _token: &str) -> Result<Option<String>, ApplicationError> {
            Ok(None)
        }

        async fn revoke(&self, token: &str) -> Result<(), ApplicationError> {
            self.revoked_tokens.lock().unwrap().push(token.to_string());
            Ok(())
        }

        async fn revoke_all_for_user(&self, _user_id: &str) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn logout_revokes_refresh_token() {
        let refresh_token_store = std::sync::Arc::new(FakeRefreshTokenStore::default());
        let use_case = Logout::new(refresh_token_store.clone());

        use_case
            .execute(LogoutInput {
                refresh_token: "refresh-user-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            refresh_token_store
                .revoked_tokens
                .lock()
                .unwrap()
                .as_slice(),
            &["refresh-user-1".to_string()]
        );
    }
}
