use crate::{
    BrandQueryService, BrandReadRepository, Clock, CreateBrand, CreateStore,
    EnsureDefaultOrganization, IdGenerator, OrganizationUnitOfWorkFactory, StoreQueryService,
    StoreReadRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct OrganizationModule {
    create_brand: Arc<CreateBrand>,
    create_store: Arc<CreateStore>,
    brand_queries: Arc<BrandQueryService>,
    store_queries: Arc<StoreQueryService>,
    ensure_default_organization: Arc<EnsureDefaultOrganization>,
}

impl OrganizationModule {
    pub fn new(
        unit_of_work_factory: Arc<dyn OrganizationUnitOfWorkFactory>,
        brand_read_repository: Arc<dyn BrandReadRepository>,
        store_read_repository: Arc<dyn StoreReadRepository>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        let create_brand = Arc::new(CreateBrand::new(
            unit_of_work_factory.clone(),
            clock.clone(),
            id_generator.clone(),
        ));
        let create_store = Arc::new(CreateStore::new(
            unit_of_work_factory.clone(),
            clock.clone(),
            id_generator.clone(),
        ));
        let brand_queries = Arc::new(BrandQueryService::new(brand_read_repository));
        let store_queries = Arc::new(StoreQueryService::new(store_read_repository));

        Self {
            ensure_default_organization: Arc::new(EnsureDefaultOrganization::new(
                unit_of_work_factory,
                clock,
                id_generator,
            )),
            create_brand,
            create_store,
            brand_queries,
            store_queries,
        }
    }

    pub fn create_brand(&self) -> &Arc<CreateBrand> {
        &self.create_brand
    }

    pub fn create_store(&self) -> &Arc<CreateStore> {
        &self.create_store
    }

    pub fn brand_queries(&self) -> &Arc<BrandQueryService> {
        &self.brand_queries
    }

    pub fn store_queries(&self) -> &Arc<StoreQueryService> {
        &self.store_queries
    }

    pub fn ensure_default_organization(&self) -> &Arc<EnsureDefaultOrganization> {
        &self.ensure_default_organization
    }
}
