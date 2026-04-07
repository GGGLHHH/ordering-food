use crate::db_roles::{DbGlobalRole, DbStoreRole};
use async_trait::async_trait;
use ordering_food_access_application::{AccessGrantRepository, ApplicationError};
use ordering_food_access_domain::AccessRole;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxAccessGrantRepository {
    pool: PgPool,
}

impl SqlxAccessGrantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(value)
            .map_err(|_| ApplicationError::unexpected(format!("{field} must be a valid UUID")))
    }
}

#[async_trait]
impl AccessGrantRepository for SqlxAccessGrantRepository {
    async fn get_platform_roles(
        &self,
        subject_id: &str,
    ) -> Result<Vec<AccessRole>, ApplicationError> {
        let subject_id = Self::parse_uuid(subject_id, "subject id")?;
        let rows = sqlx::query(
            r#"
            SELECT role
            FROM access.subject_global_roles
            WHERE subject_id = $1
            ORDER BY role
            "#,
        )
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query platform roles", error)
        })?;

        rows.into_iter()
            .map(|row| {
                row.try_get::<DbGlobalRole, _>("role")
                    .map(AccessRole::from)
                    .map_err(|error| {
                        ApplicationError::unexpected_with_source(
                            "failed to decode platform role",
                            error,
                        )
                    })
            })
            .collect()
    }

    async fn get_store_roles(
        &self,
        subject_id: &str,
        store_id: &str,
    ) -> Result<Vec<AccessRole>, ApplicationError> {
        let subject_id = Self::parse_uuid(subject_id, "subject id")?;
        let store_id = Self::parse_uuid(store_id, "store id")?;
        let rows = sqlx::query(
            r#"
            SELECT role
            FROM access.store_memberships
            WHERE subject_id = $1 AND store_id = $2
            ORDER BY role
            "#,
        )
        .bind(subject_id)
        .bind(store_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query store roles", error)
        })?;

        rows.into_iter()
            .map(|row| {
                row.try_get::<DbStoreRole, _>("role")
                    .map(AccessRole::from)
                    .map_err(|error| {
                        ApplicationError::unexpected_with_source(
                            "failed to decode store role",
                            error,
                        )
                    })
            })
            .collect()
    }
}
