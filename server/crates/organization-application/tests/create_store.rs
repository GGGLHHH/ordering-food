use async_trait::async_trait;
use ordering_food_organization_application::{
    ApplicationError, CreateBrand, CreateBrandInput, CreateStore, CreateStoreInput, IdGenerator,
    OrganizationUnitOfWork, OrganizationUnitOfWorkFactory, StoreQueryService, StoreReadRepository,
};
use ordering_food_organization_domain::{Brand, BrandId, Store, StoreId};
use ordering_food_organization_published::StoreSummary;
use ordering_food_shared_kernel::Timestamp;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use time::macros::datetime;

#[tokio::test]
async fn create_store_requires_existing_brand() {
    let use_case = test_create_store_use_case_without_brands();

    let error = use_case
        .execute(CreateStoreInput {
            brand_id: "brand-missing".to_string(),
            slug: "demo-kitchen".to_string(),
            name: "Demo Kitchen".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::NotFound { ref message } if message == "brand was not found")
    );
}

#[tokio::test]
async fn create_brand_persists_brand() {
    let unit_of_work_factory = Arc::new(FakeUnitOfWorkFactory::default());
    let use_case = CreateBrand::new(
        unit_of_work_factory.clone(),
        Arc::new(FixedClock::default()),
        Arc::new(FixedIdGenerator),
    );

    let brand = use_case
        .execute(CreateBrandInput {
            brand_id: None,
            slug: "ordering-food".to_string(),
            name: "Ordering Food".to_string(),
            status: "active".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(brand.id().as_str(), "brand-1");
    assert_eq!(brand.slug(), "ordering-food");
    assert_eq!(
        unit_of_work_factory
            .state
            .lock()
            .unwrap()
            .brands
            .get("brand-1")
            .unwrap()
            .name(),
        "Ordering Food"
    );
}

#[tokio::test]
async fn create_store_persists_store_when_brand_exists() {
    let unit_of_work_factory = Arc::new(FakeUnitOfWorkFactory::default());
    unit_of_work_factory.state.lock().unwrap().brands.insert(
        "brand-1".to_string(),
        Brand::create(
            BrandId::new("brand-1"),
            "ordering-food",
            "Ordering Food",
            ordering_food_organization_domain::OrganizationStatus::Active,
            datetime!(2026-04-05 08:00 UTC),
        )
        .unwrap(),
    );
    let use_case = CreateStore::new(
        unit_of_work_factory.clone(),
        Arc::new(FixedClock::default()),
        Arc::new(FixedIdGenerator),
    );

    let store = use_case
        .execute(CreateStoreInput {
            brand_id: "brand-1".to_string(),
            slug: "demo-kitchen".to_string(),
            name: "Demo Kitchen".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(store.id().as_str(), "store-1");
    assert_eq!(store.brand_id().as_str(), "brand-1");
    assert_eq!(
        unit_of_work_factory
            .state
            .lock()
            .unwrap()
            .stores
            .get("store-1")
            .unwrap()
            .name(),
        "Demo Kitchen"
    );
}

#[tokio::test]
async fn create_store_rolls_back_when_store_validation_fails() {
    let unit_of_work_factory = Arc::new(FakeUnitOfWorkFactory::default());
    unit_of_work_factory.state.lock().unwrap().brands.insert(
        "brand-1".to_string(),
        Brand::create(
            BrandId::new("brand-1"),
            "ordering-food",
            "Ordering Food",
            ordering_food_organization_domain::OrganizationStatus::Active,
            datetime!(2026-04-05 08:00 UTC),
        )
        .unwrap(),
    );
    let use_case = CreateStore::new(
        unit_of_work_factory.clone(),
        Arc::new(FixedClock::default()),
        Arc::new(FixedIdGenerator),
    );

    let error = use_case
        .execute(CreateStoreInput {
            brand_id: "brand-1".to_string(),
            slug: "demo-kitchen".to_string(),
            name: "Demo Kitchen".to_string(),
            currency_code: "RMBB".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Validation { .. }));
    assert_eq!(*unit_of_work_factory.rollback_count.lock().unwrap(), 1);
    assert_eq!(*unit_of_work_factory.commit_count.lock().unwrap(), 0);
}

#[tokio::test]
async fn create_store_rolls_back_when_brand_lookup_fails() {
    let unit_of_work_factory = Arc::new(FakeUnitOfWorkFactory::default());
    unit_of_work_factory.state.lock().unwrap().fail_brand_lookup = true;
    let use_case = CreateStore::new(
        unit_of_work_factory.clone(),
        Arc::new(FixedClock::default()),
        Arc::new(FixedIdGenerator),
    );

    let error = use_case
        .execute(CreateStoreInput {
            brand_id: "brand-1".to_string(),
            slug: "demo-kitchen".to_string(),
            name: "Demo Kitchen".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::Unexpected { .. }));
    assert_eq!(*unit_of_work_factory.rollback_count.lock().unwrap(), 1);
    assert_eq!(*unit_of_work_factory.commit_count.lock().unwrap(), 0);
}

#[tokio::test]
async fn store_queries_return_active_store_summary() {
    let summary = StoreSummary {
        store_id: "store-1".to_string(),
        brand_id: "brand-1".to_string(),
        slug: "demo-kitchen".to_string(),
        name: "Demo Kitchen".to_string(),
        currency_code: "CNY".to_string(),
        timezone: "Asia/Shanghai".to_string(),
        status: "active".to_string(),
    };
    let queries = StoreQueryService::new(Arc::new(FakeStoreReadRepository {
        active_store: Mutex::new(Some(summary.clone())),
    }));

    let active_store = queries.get_active().await.unwrap();

    assert_eq!(active_store, Some(summary));
}

fn test_create_store_use_case_without_brands() -> CreateStore {
    CreateStore::new(
        Arc::new(FakeUnitOfWorkFactory::default()),
        Arc::new(FixedClock::default()),
        Arc::new(FixedIdGenerator),
    )
}

#[derive(Default)]
struct FakeUnitOfWorkFactory {
    state: Arc<Mutex<FakeOrganizationState>>,
    begin_count: Arc<Mutex<u32>>,
    commit_count: Arc<Mutex<u32>>,
    rollback_count: Arc<Mutex<u32>>,
}

#[async_trait]
impl OrganizationUnitOfWorkFactory for FakeUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn OrganizationUnitOfWork>, ApplicationError> {
        *self.begin_count.lock().unwrap() += 1;
        Ok(Box::new(FakeUnitOfWork {
            state: self.state.clone(),
            commit_count: self.commit_count.clone(),
            rollback_count: self.rollback_count.clone(),
        }))
    }
}

struct FakeUnitOfWork {
    state: Arc<Mutex<FakeOrganizationState>>,
    commit_count: Arc<Mutex<u32>>,
    rollback_count: Arc<Mutex<u32>>,
}

#[async_trait]
impl OrganizationUnitOfWork for FakeUnitOfWork {
    async fn find_brand_by_id(
        &mut self,
        brand_id: &BrandId,
    ) -> Result<Option<Brand>, ApplicationError> {
        let state = self.state.lock().unwrap();
        if state.fail_brand_lookup {
            return Err(ApplicationError::unexpected("brand lookup failed"));
        }
        Ok(state.brands.get(brand_id.as_str()).cloned())
    }

    async fn insert_brand(&mut self, brand: &Brand) -> Result<(), ApplicationError> {
        self.state
            .lock()
            .unwrap()
            .brands
            .insert(brand.id().as_str().to_string(), brand.clone());
        Ok(())
    }

    async fn find_store_by_brand_slug(
        &mut self,
        brand_id: &BrandId,
        slug: &str,
    ) -> Result<Option<Store>, ApplicationError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .stores
            .values()
            .find(|store| store.brand_id() == brand_id && store.slug() == slug)
            .cloned())
    }

    async fn insert_store(&mut self, store: &Store) -> Result<(), ApplicationError> {
        self.state
            .lock()
            .unwrap()
            .stores
            .insert(store.id().as_str().to_string(), store.clone());
        Ok(())
    }

    async fn update_store(&mut self, store: &Store) -> Result<(), ApplicationError> {
        self.state
            .lock()
            .unwrap()
            .stores
            .insert(store.id().as_str().to_string(), store.clone());
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        *self.commit_count.lock().unwrap() += 1;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        *self.rollback_count.lock().unwrap() += 1;
        Ok(())
    }
}

#[derive(Default)]
struct FakeOrganizationState {
    brands: HashMap<String, Brand>,
    stores: HashMap<String, Store>,
    fail_brand_lookup: bool,
}

struct FixedClock {
    now: Timestamp,
}

impl Default for FixedClock {
    fn default() -> Self {
        Self {
            now: datetime!(2026-04-05 08:00 UTC),
        }
    }
}

impl ordering_food_platform_kernel::Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

struct FixedIdGenerator;

impl IdGenerator for FixedIdGenerator {
    fn next_brand_id(&self) -> BrandId {
        BrandId::new("brand-1")
    }

    fn next_store_id(&self) -> StoreId {
        StoreId::new("store-1")
    }
}

struct FakeStoreReadRepository {
    active_store: Mutex<Option<StoreSummary>>,
}

#[async_trait]
impl StoreReadRepository for FakeStoreReadRepository {
    async fn get_active(&self) -> Result<Option<StoreSummary>, ApplicationError> {
        Ok(self.active_store.lock().unwrap().clone())
    }

    async fn get_by_id(
        &self,
        store_id: &StoreId,
    ) -> Result<Option<StoreSummary>, ApplicationError> {
        Ok(self
            .active_store
            .lock()
            .unwrap()
            .clone()
            .filter(|store| store.store_id == store_id.as_str()))
    }
}
