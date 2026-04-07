use crate::{ApplicationError, BrandRef, StoreSummary};
use async_trait::async_trait;
use ordering_food_organization_domain::{Brand, BrandId, Store, StoreId};
pub use ordering_food_platform_kernel::Clock;
use std::sync::Arc;

pub trait IdGenerator: Send + Sync {
    fn next_brand_id(&self) -> BrandId;
    fn next_store_id(&self) -> StoreId;
}

#[async_trait]
pub trait OrganizationUnitOfWork: Send {
    async fn find_brand_by_id(
        &mut self,
        brand_id: &BrandId,
    ) -> Result<Option<Brand>, ApplicationError>;

    async fn insert_brand(&mut self, brand: &Brand) -> Result<(), ApplicationError>;

    async fn find_store_by_brand_slug(
        &mut self,
        brand_id: &BrandId,
        slug: &str,
    ) -> Result<Option<Store>, ApplicationError>;

    async fn insert_store(&mut self, store: &Store) -> Result<(), ApplicationError>;

    async fn update_store(&mut self, store: &Store) -> Result<(), ApplicationError>;

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError>;
    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait OrganizationUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn OrganizationUnitOfWork>, ApplicationError>;
}

#[async_trait]
pub trait BrandReadRepository: Send + Sync {
    async fn get_by_id(&self, brand_id: &BrandId) -> Result<Option<BrandRef>, ApplicationError>;
}

#[async_trait]
pub trait StoreReadRepository: Send + Sync {
    async fn get_active(&self) -> Result<Option<StoreSummary>, ApplicationError>;
    async fn get_by_id(&self, store_id: &StoreId)
    -> Result<Option<StoreSummary>, ApplicationError>;
}

#[derive(Clone)]
pub struct BrandQueryService {
    repository: Arc<dyn BrandReadRepository>,
}

impl BrandQueryService {
    pub fn new(repository: Arc<dyn BrandReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_by_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandRef>, ApplicationError> {
        self.repository.get_by_id(&BrandId::new(brand_id)).await
    }
}

#[derive(Clone)]
pub struct StoreQueryService {
    repository: Arc<dyn StoreReadRepository>,
}

impl StoreQueryService {
    pub fn new(repository: Arc<dyn StoreReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_active(&self) -> Result<Option<StoreSummary>, ApplicationError> {
        self.repository.get_active().await
    }

    pub async fn get_by_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreSummary>, ApplicationError> {
        self.repository.get_by_id(&StoreId::new(store_id)).await
    }
}
