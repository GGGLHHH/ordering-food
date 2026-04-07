use crate::{
    brand_repository::{find_brand_by_id, insert_brand},
    store_repository::{find_store_by_brand_slug, insert_store, update_store},
};
use async_trait::async_trait;
use ordering_food_organization_application::{
    ApplicationError, OrganizationUnitOfWork, OrganizationUnitOfWorkFactory,
};
use ordering_food_organization_domain::{Brand, BrandId, Store};
use sqlx::{PgPool, Postgres, Transaction};

pub struct SqlxOrganizationUnitOfWork {
    transaction: Transaction<'static, Postgres>,
}

impl SqlxOrganizationUnitOfWork {
    pub fn new(transaction: Transaction<'static, Postgres>) -> Self {
        Self { transaction }
    }

    async fn commit_inner(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self { transaction } = *self;
        transaction.commit().await.map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to commit organization transaction",
                error,
            )
        })
    }

    async fn rollback_inner(self: Box<Self>) -> Result<(), ApplicationError> {
        let Self { transaction } = *self;
        transaction.rollback().await.map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to rollback organization transaction",
                error,
            )
        })
    }
}

#[async_trait]
impl OrganizationUnitOfWork for SqlxOrganizationUnitOfWork {
    async fn find_brand_by_id(
        &mut self,
        brand_id: &BrandId,
    ) -> Result<Option<Brand>, ApplicationError> {
        find_brand_by_id(&mut self.transaction, brand_id).await
    }

    async fn insert_brand(&mut self, brand: &Brand) -> Result<(), ApplicationError> {
        insert_brand(&mut self.transaction, brand).await
    }

    async fn find_store_by_brand_slug(
        &mut self,
        brand_id: &BrandId,
        slug: &str,
    ) -> Result<Option<Store>, ApplicationError> {
        find_store_by_brand_slug(&mut self.transaction, brand_id, slug).await
    }

    async fn insert_store(&mut self, store: &Store) -> Result<(), ApplicationError> {
        insert_store(&mut self.transaction, store).await
    }

    async fn update_store(&mut self, store: &Store) -> Result<(), ApplicationError> {
        update_store(&mut self.transaction, store).await
    }

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        self.commit_inner().await
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        self.rollback_inner().await
    }
}

#[derive(Clone)]
pub struct SqlxOrganizationUnitOfWorkFactory {
    pool: PgPool,
}

impl SqlxOrganizationUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationUnitOfWorkFactory for SqlxOrganizationUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn OrganizationUnitOfWork>, ApplicationError> {
        let transaction = self.pool.begin().await.map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to begin organization transaction",
                error,
            )
        })?;

        Ok(Box::new(SqlxOrganizationUnitOfWork::new(transaction)))
    }
}
