use ordering_food_shared::error::AppError;
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LiveResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReadyResponse {
    pub status: String,
    pub checks: DependencyChecks,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DependencyChecks {
    pub postgres: String,
    pub redis: String,
}

#[async_trait]
pub trait ReadinessProbe: Send + Sync {
    async fn check(&self) -> Result<DependencyChecks, AppError>;
}

#[derive(Clone)]
pub struct RuntimeReadiness {
    pg_pool: PgPool,
    redis_client: redis::Client,
}

impl RuntimeReadiness {
    pub fn new(pg_pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            pg_pool,
            redis_client,
        }
    }
}

#[async_trait]
impl ReadinessProbe for RuntimeReadiness {
    async fn check(&self) -> Result<DependencyChecks, AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.pg_pool)
            .await
            .map_err(|error| {
                AppError::dependency_unavailable_with_source(
                    "postgres readiness check failed",
                    error,
                )
            })?;

        let mut connection = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| {
                AppError::dependency_unavailable_with_source("redis connection failed", error)
            })?;

        let pong = ping_redis(&mut connection).await?;
        if pong != "PONG" {
            return Err(AppError::dependency_unavailable(format!(
                "redis ping returned unexpected response: {pong}"
            )));
        }

        Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        })
    }
}

async fn ping_redis(connection: &mut MultiplexedConnection) -> Result<String, AppError> {
    redis::cmd("PING")
        .query_async(connection)
        .await
        .map_err(|error| AppError::dependency_unavailable_with_source("redis ping failed", error))
}
