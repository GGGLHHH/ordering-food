use crate::{
    ActiveCatalogQueryService, AttachStoreCatalog, BootstrapBrandCatalog, BootstrapDefaultCatalog,
    BrandCatalogQueryService, BrandCatalogReadRepository, BrandCatalogRepository,
    CategoryQueryService, CategoryReadRepository, CategoryRepository, Clock, CreateCategory,
    CreateItem, IdGenerator, ItemQueryService, ItemReadRepository, ItemRepository,
    OrganizationScopeReader, StoreCatalogQueryService, StoreCatalogReadRepository,
    StoreCatalogRepository, StoreItemListingQueryService, StoreItemListingReadRepository,
    StoreItemListingRepository, TransactionManager, UpsertStoreItemListing,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct CatalogModule {
    bootstrap_default_catalog: Arc<BootstrapDefaultCatalog>,
    bootstrap_brand_catalog: Arc<BootstrapBrandCatalog>,
    attach_store_catalog: Arc<AttachStoreCatalog>,
    create_category: Arc<CreateCategory>,
    create_item: Arc<CreateItem>,
    upsert_store_item_listing: Arc<UpsertStoreItemListing>,
    active_catalog_queries: Arc<ActiveCatalogQueryService>,
    brand_catalog_queries: Arc<BrandCatalogQueryService>,
    store_catalog_queries: Arc<StoreCatalogQueryService>,
    category_queries: Arc<CategoryQueryService>,
    item_queries: Arc<ItemQueryService>,
    store_item_listing_queries: Arc<StoreItemListingQueryService>,
}

impl CatalogModule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_scope_reader: Arc<dyn OrganizationScopeReader>,
        brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
        store_catalog_repository: Arc<dyn StoreCatalogRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        item_repository: Arc<dyn ItemRepository>,
        store_item_listing_repository: Arc<dyn StoreItemListingRepository>,
        brand_catalog_read_repository: Arc<dyn BrandCatalogReadRepository>,
        store_catalog_read_repository: Arc<dyn StoreCatalogReadRepository>,
        category_read_repository: Arc<dyn CategoryReadRepository>,
        item_read_repository: Arc<dyn ItemReadRepository>,
        store_item_listing_read_repository: Arc<dyn StoreItemListingReadRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        let bootstrap_brand_catalog = Arc::new(BootstrapBrandCatalog::new(
            organization_scope_reader.clone(),
            brand_catalog_repository.clone(),
            transaction_manager.clone(),
            clock.clone(),
            id_generator.clone(),
        ));
        let attach_store_catalog = Arc::new(AttachStoreCatalog::new(
            organization_scope_reader.clone(),
            store_catalog_repository.clone(),
            transaction_manager.clone(),
            clock.clone(),
            id_generator.clone(),
        ));
        let create_category = Arc::new(CreateCategory::new(
            brand_catalog_repository.clone(),
            category_repository.clone(),
            transaction_manager.clone(),
            clock.clone(),
            id_generator.clone(),
        ));
        let create_item = Arc::new(CreateItem::new(
            brand_catalog_repository,
            category_repository,
            item_repository.clone(),
            transaction_manager.clone(),
            clock.clone(),
            id_generator.clone(),
        ));
        let upsert_store_item_listing = Arc::new(UpsertStoreItemListing::new(
            store_catalog_repository,
            item_repository,
            store_item_listing_repository,
            transaction_manager,
            clock,
        ));
        let brand_catalog_queries =
            Arc::new(BrandCatalogQueryService::new(brand_catalog_read_repository));
        let store_catalog_queries =
            Arc::new(StoreCatalogQueryService::new(store_catalog_read_repository));
        let category_queries = Arc::new(CategoryQueryService::new(category_read_repository));
        let item_queries = Arc::new(ItemQueryService::new(item_read_repository));
        let store_item_listing_queries = Arc::new(StoreItemListingQueryService::new(
            store_item_listing_read_repository,
        ));
        let active_catalog_queries = Arc::new(ActiveCatalogQueryService::new(
            organization_scope_reader.clone(),
            brand_catalog_queries.clone(),
            store_catalog_queries.clone(),
        ));

        Self {
            bootstrap_default_catalog: Arc::new(BootstrapDefaultCatalog::new(
                bootstrap_brand_catalog.clone(),
                attach_store_catalog.clone(),
                create_category.clone(),
                create_item.clone(),
                upsert_store_item_listing.clone(),
                brand_catalog_queries.clone(),
                store_catalog_queries.clone(),
                category_queries.clone(),
                item_queries.clone(),
            )),
            bootstrap_brand_catalog,
            attach_store_catalog,
            create_category,
            create_item,
            upsert_store_item_listing,
            active_catalog_queries,
            brand_catalog_queries,
            store_catalog_queries,
            category_queries,
            item_queries,
            store_item_listing_queries,
        }
    }

    pub fn bootstrap_default_catalog(&self) -> &Arc<BootstrapDefaultCatalog> {
        &self.bootstrap_default_catalog
    }

    pub fn bootstrap_brand_catalog(&self) -> &Arc<BootstrapBrandCatalog> {
        &self.bootstrap_brand_catalog
    }

    pub fn attach_store_catalog(&self) -> &Arc<AttachStoreCatalog> {
        &self.attach_store_catalog
    }

    pub fn create_category(&self) -> &Arc<CreateCategory> {
        &self.create_category
    }

    pub fn create_item(&self) -> &Arc<CreateItem> {
        &self.create_item
    }

    pub fn upsert_store_item_listing(&self) -> &Arc<UpsertStoreItemListing> {
        &self.upsert_store_item_listing
    }

    pub fn active_catalog_queries(&self) -> &Arc<ActiveCatalogQueryService> {
        &self.active_catalog_queries
    }

    pub fn brand_catalog_queries(&self) -> &Arc<BrandCatalogQueryService> {
        &self.brand_catalog_queries
    }

    pub fn store_catalog_queries(&self) -> &Arc<StoreCatalogQueryService> {
        &self.store_catalog_queries
    }

    pub fn category_queries(&self) -> &Arc<CategoryQueryService> {
        &self.category_queries
    }

    pub fn item_queries(&self) -> &Arc<ItemQueryService> {
        &self.item_queries
    }

    pub fn store_item_listing_queries(&self) -> &Arc<StoreItemListingQueryService> {
        &self.store_item_listing_queries
    }
}
