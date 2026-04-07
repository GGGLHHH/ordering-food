use crate::{
    ApplicationError, BrandCatalogReadModel, CategoryReadModel, ItemReadModel,
    StoreCatalogReadModel, StoreItemListingReadModel,
};
use async_trait::async_trait;
use ordering_food_catalog_domain::{
    BrandCatalog, BrandCatalogId, BrandId, Category, CategoryId, Item, ItemId, StoreCatalog,
    StoreCatalogId, StoreId, StoreItemListing,
};
use ordering_food_organization_published::{BrandRef, StoreSummary};
pub use ordering_food_platform_kernel::Clock;
use std::{any::Any, sync::Arc};

pub trait IdGenerator: Send + Sync {
    fn next_brand_catalog_id(&self) -> BrandCatalogId;
    fn next_store_catalog_id(&self) -> StoreCatalogId;
    fn next_category_id(&self) -> CategoryId;
    fn next_item_id(&self) -> ItemId;
}

pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError>;
    async fn commit(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
    async fn rollback(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait OrganizationScopeReader: Send + Sync {
    async fn get_active_store(&self) -> Result<Option<StoreSummary>, ApplicationError>;
    async fn get_brand(&self, brand_id: &str) -> Result<Option<BrandRef>, ApplicationError>;
    async fn get_store_scope(
        &self,
        brand_id: &str,
        store_id: &str,
    ) -> Result<Option<StoreSummary>, ApplicationError>;
}

#[async_trait]
pub trait BrandCatalogRepository: Send + Sync {
    async fn find_by_brand_id(
        &self,
        tx: &mut dyn TransactionContext,
        brand_id: &BrandId,
    ) -> Result<Option<BrandCatalog>, ApplicationError>;

    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        brand_catalog_id: &BrandCatalogId,
    ) -> Result<Option<BrandCatalog>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        brand_catalog: &BrandCatalog,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait StoreCatalogRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_catalog_id: &StoreCatalogId,
    ) -> Result<Option<StoreCatalog>, ApplicationError>;

    async fn find_by_store_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_id: &StoreId,
    ) -> Result<Option<StoreCatalog>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        store_catalog: &StoreCatalog,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        category_id: &CategoryId,
    ) -> Result<Option<Category>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        category: &Category,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        item_id: &ItemId,
    ) -> Result<Option<Item>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        item: &Item,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait StoreItemListingRepository: Send + Sync {
    async fn upsert(
        &self,
        tx: &mut dyn TransactionContext,
        listing: &StoreItemListing,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait BrandCatalogReadRepository: Send + Sync {
    async fn find_by_id(
        &self,
        brand_catalog_id: &BrandCatalogId,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError>;

    async fn find_by_brand_id(
        &self,
        brand_id: &BrandId,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError>;
}

#[async_trait]
pub trait StoreCatalogReadRepository: Send + Sync {
    async fn find_by_id(
        &self,
        store_catalog_id: &StoreCatalogId,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError>;

    async fn find_by_store_id(
        &self,
        store_id: &StoreId,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError>;
}

#[async_trait]
pub trait CategoryReadRepository: Send + Sync {
    async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &BrandCatalogId,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError>;

    async fn find_by_slug(
        &self,
        brand_catalog_id: &BrandCatalogId,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError>;
}

#[derive(Debug, Clone, Default)]
pub struct CatalogItemListFilter {
    pub category_id: Option<String>,
}

#[async_trait]
pub trait ItemReadRepository: Send + Sync {
    async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &BrandCatalogId,
        filter: CatalogItemListFilter,
    ) -> Result<Vec<ItemReadModel>, ApplicationError>;

    async fn find_by_id(&self, item_id: &ItemId)
    -> Result<Option<ItemReadModel>, ApplicationError>;

    async fn find_by_slug(
        &self,
        brand_catalog_id: &BrandCatalogId,
        slug: &str,
    ) -> Result<Option<ItemReadModel>, ApplicationError>;
}

#[async_trait]
pub trait StoreItemListingReadRepository: Send + Sync {
    async fn find_by_item_id(
        &self,
        store_catalog_id: &StoreCatalogId,
        item_id: &ItemId,
    ) -> Result<Option<StoreItemListingReadModel>, ApplicationError>;

    async fn list_by_store_catalog_id(
        &self,
        store_catalog_id: &StoreCatalogId,
    ) -> Result<Vec<StoreItemListingReadModel>, ApplicationError>;
}

#[derive(Clone)]
pub struct BrandCatalogQueryService {
    repository: Arc<dyn BrandCatalogReadRepository>,
}

impl BrandCatalogQueryService {
    pub fn new(repository: Arc<dyn BrandCatalogReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn find_by_id(
        &self,
        brand_catalog_id: &str,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError> {
        self.repository
            .find_by_id(&BrandCatalogId::new(brand_catalog_id))
            .await
    }

    pub async fn find_by_brand_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandCatalogReadModel>, ApplicationError> {
        self.repository
            .find_by_brand_id(&BrandId::new(brand_id))
            .await
    }
}

#[derive(Clone)]
pub struct StoreCatalogQueryService {
    repository: Arc<dyn StoreCatalogReadRepository>,
}

impl StoreCatalogQueryService {
    pub fn new(repository: Arc<dyn StoreCatalogReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn find_by_id(
        &self,
        store_catalog_id: &str,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError> {
        self.repository
            .find_by_id(&StoreCatalogId::new(store_catalog_id))
            .await
    }

    pub async fn find_by_store_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreCatalogReadModel>, ApplicationError> {
        self.repository
            .find_by_store_id(&StoreId::new(store_id))
            .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveCatalogContextReadModel {
    pub store: StoreSummary,
    pub brand_catalog: BrandCatalogReadModel,
    pub store_catalog: StoreCatalogReadModel,
}

#[derive(Clone)]
pub struct ActiveCatalogQueryService {
    organization_scope_reader: Arc<dyn OrganizationScopeReader>,
    brand_catalog_queries: Arc<BrandCatalogQueryService>,
    store_catalog_queries: Arc<StoreCatalogQueryService>,
}

impl ActiveCatalogQueryService {
    pub fn new(
        organization_scope_reader: Arc<dyn OrganizationScopeReader>,
        brand_catalog_queries: Arc<BrandCatalogQueryService>,
        store_catalog_queries: Arc<StoreCatalogQueryService>,
    ) -> Self {
        Self {
            organization_scope_reader,
            brand_catalog_queries,
            store_catalog_queries,
        }
    }

    pub async fn load_active(&self) -> Result<ActiveCatalogContextReadModel, ApplicationError> {
        let store = self
            .organization_scope_reader
            .get_active_store()
            .await?
            .ok_or_else(|| ApplicationError::not_found("organization store was not found"))?;
        let brand_catalog = self
            .brand_catalog_queries
            .find_by_brand_id(&store.brand_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("brand catalog was not found"))?;
        let store_catalog = self
            .store_catalog_queries
            .find_by_store_id(&store.store_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("store catalog was not found"))?;

        Ok(ActiveCatalogContextReadModel {
            store,
            brand_catalog,
            store_catalog,
        })
    }
}

#[derive(Clone)]
pub struct CategoryQueryService {
    repository: Arc<dyn CategoryReadRepository>,
}

impl CategoryQueryService {
    pub fn new(repository: Arc<dyn CategoryReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &str,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError> {
        self.repository
            .list_by_brand_catalog_id(&BrandCatalogId::new(brand_catalog_id))
            .await
    }

    pub async fn find_by_slug(
        &self,
        brand_catalog_id: &str,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError> {
        self.repository
            .find_by_slug(&BrandCatalogId::new(brand_catalog_id), slug)
            .await
    }
}

#[derive(Clone)]
pub struct ItemQueryService {
    repository: Arc<dyn ItemReadRepository>,
}

impl ItemQueryService {
    pub fn new(repository: Arc<dyn ItemReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_by_brand_catalog_id(
        &self,
        brand_catalog_id: &str,
        filter: CatalogItemListFilter,
    ) -> Result<Vec<ItemReadModel>, ApplicationError> {
        self.repository
            .list_by_brand_catalog_id(&BrandCatalogId::new(brand_catalog_id), filter)
            .await
    }

    pub async fn find_by_id(
        &self,
        item_id: &str,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        self.repository.find_by_id(&ItemId::new(item_id)).await
    }

    pub async fn find_by_slug(
        &self,
        brand_catalog_id: &str,
        slug: &str,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        self.repository
            .find_by_slug(&BrandCatalogId::new(brand_catalog_id), slug)
            .await
    }
}

#[derive(Clone)]
pub struct StoreItemListingQueryService {
    repository: Arc<dyn StoreItemListingReadRepository>,
}

impl StoreItemListingQueryService {
    pub fn new(repository: Arc<dyn StoreItemListingReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn find_by_item_id(
        &self,
        store_catalog_id: &str,
        item_id: &str,
    ) -> Result<Option<StoreItemListingReadModel>, ApplicationError> {
        self.repository
            .find_by_item_id(
                &StoreCatalogId::new(store_catalog_id),
                &ItemId::new(item_id),
            )
            .await
    }

    pub async fn list_by_store_catalog_id(
        &self,
        store_catalog_id: &str,
    ) -> Result<Vec<StoreItemListingReadModel>, ApplicationError> {
        self.repository
            .list_by_store_catalog_id(&StoreCatalogId::new(store_catalog_id))
            .await
    }
}
