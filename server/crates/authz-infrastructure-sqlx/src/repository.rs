use crate::db_roles::{DbGlobalRole, DbStoreRole};
use async_trait::async_trait;
use ordering_food_authz_application::{ApplicationError, AuthorizationRepository};
use ordering_food_authz_domain::{GlobalRole, StoreRole};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxAuthorizationRepository {
    pool: PgPool,
}

impl SqlxAuthorizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(value)
            .map_err(|_| ApplicationError::unexpected(format!("{field} must be a valid UUID")))
    }
}

#[async_trait]
impl AuthorizationRepository for SqlxAuthorizationRepository {
    async fn get_global_roles(&self, user_id: &str) -> Result<Vec<GlobalRole>, ApplicationError> {
        let user_id = Self::parse_uuid(user_id, "user id")?;
        let rows = sqlx::query(
            r#"
            SELECT role
            FROM authz.user_global_roles
            WHERE user_id = $1
            ORDER BY role
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query global roles", error)
        })?;

        rows.into_iter()
            .map(|row| Ok(GlobalRole::from(row.get::<DbGlobalRole, _>("role"))))
            .collect()
    }

    async fn get_store_roles(
        &self,
        user_id: &str,
        store_id: &str,
    ) -> Result<Vec<StoreRole>, ApplicationError> {
        let user_id = Self::parse_uuid(user_id, "user id")?;
        let store_id = Self::parse_uuid(store_id, "store id")?;
        let rows = sqlx::query(
            r#"
            SELECT role
            FROM authz.store_memberships
            WHERE user_id = $1 AND store_id = $2
            ORDER BY role
            "#,
        )
        .bind(user_id)
        .bind(store_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query store roles", error)
        })?;

        rows.into_iter()
            .map(|row| Ok(StoreRole::from(row.get::<DbStoreRole, _>("role"))))
            .collect()
    }
}
