use crate::{
    composition::contribution::ApiNamedReadinessCheck,
    error::{AppError, ErrorDetails, FieldIssue, FieldLocation},
};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::BTreeMap;
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contexts: BTreeMap<String, String>,
}

impl DependencyChecks {
    pub fn ok(postgres: impl Into<String>, redis: impl Into<String>) -> Self {
        Self {
            postgres: postgres.into(),
            redis: redis.into(),
            contexts: BTreeMap::new(),
        }
    }
}

#[async_trait]
pub trait ReadinessProbe: Send + Sync {
    async fn check(&self) -> Result<DependencyChecks, AppError>;
}

#[derive(Clone)]
pub struct CompositeReadiness {
    pg_pool: PgPool,
    redis_client: redis::Client,
    context_checks: Vec<ApiNamedReadinessCheck>,
}

impl CompositeReadiness {
    pub fn new(
        pg_pool: PgPool,
        redis_client: redis::Client,
        context_checks: Vec<ApiNamedReadinessCheck>,
    ) -> Self {
        Self {
            pg_pool,
            redis_client,
            context_checks,
        }
    }
}

#[async_trait]
impl ReadinessProbe for CompositeReadiness {
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

        let mut contexts = BTreeMap::new();
        for check in &self.context_checks {
            check.run().await.map_err(|error| {
                AppError::dependency_unavailable(format!(
                    "context `{}` readiness check `{}` failed",
                    check.context_id, check.label
                ))
                .with_details(ErrorDetails {
                    fields: vec![FieldIssue {
                        location: FieldLocation::Body,
                        field: Some(check.context_id.to_string()),
                        reason: "context_readiness_failed".to_string(),
                        message: error.to_string(),
                    }],
                })
            })?;
            contexts.insert(check.context_id.to_string(), "ok".to_string());
        }

        Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
            contexts,
        })
    }
}

async fn ping_redis(connection: &mut MultiplexedConnection) -> Result<String, AppError> {
    redis::cmd("PING")
        .query_async(connection)
        .await
        .map_err(|error| AppError::dependency_unavailable_with_source("redis ping failed", error))
}
