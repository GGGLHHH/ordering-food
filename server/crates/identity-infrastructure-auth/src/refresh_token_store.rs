use async_trait::async_trait;
use ordering_food_identity_application::{ApplicationError, RefreshTokenStore};
use redis::AsyncCommands;

pub struct RedisRefreshTokenStore {
    client: redis::Client,
}

impl RedisRefreshTokenStore {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }

    fn key(token: &str) -> String {
        format!("rt:{token}")
    }

    fn user_key(user_id: &str) -> String {
        format!("rt_user:{user_id}")
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection, ApplicationError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis connection failed: {e}")))
    }
}

#[async_trait]
impl RefreshTokenStore for RedisRefreshTokenStore {
    async fn store(
        &self,
        token: &str,
        user_id: &str,
        ttl_seconds: u64,
    ) -> Result<(), ApplicationError> {
        let mut conn = self.connection().await?;
        let key = Self::key(token);

        conn.set_ex::<_, _, ()>(&key, user_id, ttl_seconds)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis SET failed: {e}")))?;

        // Track token in user's set for revoke_all_for_user
        let user_key = Self::user_key(user_id);
        conn.sadd::<_, _, ()>(&user_key, token)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis SADD failed: {e}")))?;
        conn.expire::<_, ()>(&user_key, ttl_seconds as i64)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis EXPIRE failed: {e}")))?;

        Ok(())
    }

    async fn lookup(&self, token: &str) -> Result<Option<String>, ApplicationError> {
        let mut conn = self.connection().await?;
        let key = Self::key(token);

        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis GET failed: {e}")))?;

        Ok(result)
    }

    async fn revoke(&self, token: &str) -> Result<(), ApplicationError> {
        let mut conn = self.connection().await?;
        let key = Self::key(token);

        // Look up user_id to clean user set
        let user_id: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis GET failed: {e}")))?;

        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis DEL failed: {e}")))?;

        if let Some(user_id) = user_id {
            let user_key = Self::user_key(&user_id);
            conn.srem::<_, _, ()>(&user_key, token)
                .await
                .map_err(|e| ApplicationError::unexpected(format!("redis SREM failed: {e}")))?;
        }

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), ApplicationError> {
        let mut conn = self.connection().await?;
        let user_key = Self::user_key(user_id);

        let tokens: Vec<String> = conn
            .smembers(&user_key)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis SMEMBERS failed: {e}")))?;

        for token in &tokens {
            let key = Self::key(token);
            conn.del::<_, ()>(&key)
                .await
                .map_err(|e| ApplicationError::unexpected(format!("redis DEL failed: {e}")))?;
        }

        conn.del::<_, ()>(&user_key)
            .await
            .map_err(|e| ApplicationError::unexpected(format!("redis DEL failed: {e}")))?;

        Ok(())
    }
}
