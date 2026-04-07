use crate::{
    credential_repository::{find_credential_by_user_id, upsert_credential},
    user_repository::{find_user_by_id, find_user_by_identity, insert_user, update_user},
};
use async_trait::async_trait;
use ordering_food_identity_application::{
    ApplicationError, IdentityUnitOfWork, IdentityUnitOfWorkFactory, StoredCredential,
};
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, User, UserId};
use ordering_food_shared_kernel::Timestamp;
use sqlx::{PgPool, Postgres, Transaction};

pub struct SqlxIdentityUnitOfWork {
    transaction: Transaction<'static, Postgres>,
}

impl SqlxIdentityUnitOfWork {
    pub fn new(transaction: Transaction<'static, Postgres>) -> Self {
        Self { transaction }
    }

    async fn commit_inner(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self { transaction } = *self;
        transaction.commit().await.map_err(|error| {
            ApplicationError::unexpected_with_source("failed to commit identity transaction", error)
        })
    }

    async fn rollback_inner(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self { transaction } = *self;
        transaction.rollback().await.map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to rollback identity transaction",
                error,
            )
        })
    }
}

#[async_trait]
impl IdentityUnitOfWork for SqlxIdentityUnitOfWork {
    async fn find_user_by_id(
        &mut self,
        user_id: &UserId,
    ) -> Result<Option<User>, ApplicationError> {
        find_user_by_id(&mut self.transaction, user_id).await
    }

    async fn find_user_by_identity(
        &mut self,
        identity_type: &IdentityType,
        identifier: &NormalizedIdentifier,
    ) -> Result<Option<User>, ApplicationError> {
        find_user_by_identity(&mut self.transaction, identity_type, identifier).await
    }

    async fn insert_user(&mut self, user: &User) -> Result<(), ApplicationError> {
        insert_user(&mut self.transaction, user).await
    }

    async fn update_user(&mut self, user: &User) -> Result<(), ApplicationError> {
        update_user(&mut self.transaction, user).await
    }

    async fn find_credential_by_user_id(
        &mut self,
        user_id: &UserId,
    ) -> Result<Option<StoredCredential>, ApplicationError> {
        find_credential_by_user_id(&mut self.transaction, user_id).await
    }

    async fn upsert_credential(
        &mut self,
        user_id: &UserId,
        password_hash: &str,
        now: Timestamp,
    ) -> Result<(), ApplicationError> {
        upsert_credential(&mut self.transaction, user_id, password_hash, now).await
    }

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        self.commit_inner().await
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        self.rollback_inner().await
    }
}

#[derive(Clone)]
pub struct SqlxIdentityUnitOfWorkFactory {
    pool: PgPool,
}

impl SqlxIdentityUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IdentityUnitOfWorkFactory for SqlxIdentityUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn IdentityUnitOfWork>, ApplicationError> {
        let transaction = self.pool.begin().await.map_err(|error| {
            ApplicationError::unexpected_with_source("failed to begin identity transaction", error)
        })?;

        Ok(Box::new(SqlxIdentityUnitOfWork::new(transaction)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordering_food_identity_domain::{User, UserProfile};
    use sqlx::{PgPool, types::time::OffsetDateTime};
    use uuid::Uuid;

    #[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
    async fn commit_persists_changes(pool: PgPool) {
        let factory = SqlxIdentityUnitOfWorkFactory::new(pool.clone());
        let user_id = Uuid::now_v7();
        let user = User::create(
            UserId::new(user_id.to_string()),
            UserProfile::new("Alice", None, None, None).unwrap(),
            OffsetDateTime::now_utc(),
        );

        let mut unit_of_work = factory.begin().await.unwrap();
        unit_of_work.insert_user(&user).await.unwrap();
        unit_of_work.commit().await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity.users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        sqlx::query("DELETE FROM identity.users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
    async fn rollback_discards_changes(pool: PgPool) {
        let factory = SqlxIdentityUnitOfWorkFactory::new(pool.clone());
        let user_id = Uuid::now_v7();
        let user = User::create(
            UserId::new(user_id.to_string()),
            UserProfile::new("Alice", None, None, None).unwrap(),
            OffsetDateTime::now_utc(),
        );

        let mut unit_of_work = factory.begin().await.unwrap();
        unit_of_work.insert_user(&user).await.unwrap();
        unit_of_work.rollback().await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity.users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
