use crate::{
    ApplicationError, Clock, IdGenerator, OrganizationUnitOfWork, OrganizationUnitOfWorkFactory,
    StoreQueryService,
};
use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus, Store};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureDefaultOrganizationInput {
    pub brand_id: String,
    pub brand_slug: String,
    pub brand_name: String,
    pub brand_status: String,
    pub store_slug: String,
    pub store_name: String,
    pub store_currency_code: String,
    pub store_timezone: String,
    pub store_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsureDefaultOrganizationOutcome {
    Skipped { store_id: String, slug: String },
    CreatedStore { store_id: String, brand_id: String },
    CreatedBrandAndStore { store_id: String, brand_id: String },
    RecoveredStore { store_id: String, brand_id: String },
}

pub struct EnsureDefaultOrganization {
    unit_of_work_factory: Arc<dyn OrganizationUnitOfWorkFactory>,
    store_queries: Arc<StoreQueryService>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl EnsureDefaultOrganization {
    pub fn new(
        unit_of_work_factory: Arc<dyn OrganizationUnitOfWorkFactory>,
        store_queries: Arc<StoreQueryService>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            unit_of_work_factory,
            store_queries,
            clock,
            id_generator,
        }
    }

    pub async fn execute(
        &self,
        input: EnsureDefaultOrganizationInput,
    ) -> Result<EnsureDefaultOrganizationOutcome, ApplicationError> {
        if let Some(outcome) = self.current_active_store_outcome().await? {
            return Ok(outcome);
        }

        let brand_id = BrandId::new(input.brand_id.clone());
        let normalized_store_slug = Store::normalize_slug(&input.store_slug)?;
        let store_status = OrganizationStatus::parse(&input.store_status)?;
        let mut unit_of_work = self.unit_of_work_factory.begin().await?;

        match self
            .resolve_exact_store_in_transaction(
                unit_of_work.as_mut(),
                &brand_id,
                &normalized_store_slug,
                &input.store_name,
                &input.store_currency_code,
                &input.store_timezone,
            )
            .await?
        {
            ExactStoreOutcome::Skipped(outcome) => {
                unit_of_work.rollback().await?;
                return Ok(outcome);
            }
            ExactStoreOutcome::Recovered(outcome) => {
                unit_of_work.commit().await?;
                return Ok(outcome);
            }
            ExactStoreOutcome::Missing => {}
        }

        let brand_was_created = match unit_of_work.find_brand_by_id(&brand_id).await {
            Ok(Some(_)) => false,
            Ok(None) => {
                let brand = Brand::create(
                    brand_id.clone(),
                    input.brand_slug.clone(),
                    input.brand_name.clone(),
                    OrganizationStatus::parse(&input.brand_status)?,
                    self.clock.now(),
                )?;
                match unit_of_work.insert_brand(&brand).await {
                    Ok(()) => true,
                    Err(error @ ApplicationError::Conflict { .. }) => {
                        unit_of_work.rollback().await?;
                        return self.reconcile_after_conflict(&input, true, error).await;
                    }
                    Err(error) => {
                        unit_of_work.rollback().await?;
                        return Err(error);
                    }
                }
            }
            Err(error) => {
                unit_of_work.rollback().await?;
                return Err(error);
            }
        };

        let store = match Store::create(
            self.id_generator.next_store_id(),
            brand_id.clone(),
            input.store_slug.clone(),
            input.store_name.clone(),
            input.store_currency_code.clone(),
            input.store_timezone.clone(),
            store_status,
            self.clock.now(),
        ) {
            Ok(store) => store,
            Err(error) => {
                unit_of_work.rollback().await?;
                return Err(error.into());
            }
        };

        match unit_of_work.insert_store(&store).await {
            Ok(()) => {
                unit_of_work.commit().await?;
                Ok(if brand_was_created {
                    EnsureDefaultOrganizationOutcome::CreatedBrandAndStore {
                        store_id: store.id().as_str().to_string(),
                        brand_id: store.brand_id().as_str().to_string(),
                    }
                } else {
                    EnsureDefaultOrganizationOutcome::CreatedStore {
                        store_id: store.id().as_str().to_string(),
                        brand_id: store.brand_id().as_str().to_string(),
                    }
                })
            }
            Err(error @ ApplicationError::Conflict { .. }) => {
                unit_of_work.rollback().await?;
                self.reconcile_after_conflict(&input, brand_was_created, error)
                    .await
            }
            Err(error) => {
                unit_of_work.rollback().await?;
                Err(error)
            }
        }
    }

    async fn current_active_store_outcome(
        &self,
    ) -> Result<Option<EnsureDefaultOrganizationOutcome>, ApplicationError> {
        Ok(self.store_queries.get_active().await?.map(|store| {
            EnsureDefaultOrganizationOutcome::Skipped {
                store_id: store.store_id,
                slug: store.slug,
            }
        }))
    }

    async fn reconcile_after_conflict(
        &self,
        input: &EnsureDefaultOrganizationInput,
        brand_was_created: bool,
        error: ApplicationError,
    ) -> Result<EnsureDefaultOrganizationOutcome, ApplicationError> {
        if let Some(outcome) = self.current_active_store_outcome().await? {
            return Ok(outcome);
        }

        let brand_id = BrandId::new(input.brand_id.clone());
        let normalized_store_slug = Store::normalize_slug(&input.store_slug)?;
        let mut unit_of_work = self.unit_of_work_factory.begin().await?;
        match self
            .resolve_exact_store_in_transaction(
                unit_of_work.as_mut(),
                &brand_id,
                &normalized_store_slug,
                &input.store_name,
                &input.store_currency_code,
                &input.store_timezone,
            )
            .await?
        {
            ExactStoreOutcome::Skipped(outcome) => {
                unit_of_work.rollback().await?;
                return Ok(outcome);
            }
            ExactStoreOutcome::Recovered(outcome) => {
                unit_of_work.commit().await?;
                return Ok(outcome);
            }
            ExactStoreOutcome::Missing => {}
        }

        let brand_exists = match unit_of_work.find_brand_by_id(&brand_id).await {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(repo_error) => {
                unit_of_work.rollback().await?;
                return Err(repo_error);
            }
        };
        if !brand_exists {
            unit_of_work.rollback().await?;
            return Err(error);
        }

        let store = match Store::create(
            self.id_generator.next_store_id(),
            brand_id.clone(),
            input.store_slug.clone(),
            input.store_name.clone(),
            input.store_currency_code.clone(),
            input.store_timezone.clone(),
            OrganizationStatus::parse(&input.store_status)?,
            self.clock.now(),
        ) {
            Ok(store) => store,
            Err(domain_error) => {
                unit_of_work.rollback().await?;
                return Err(domain_error.into());
            }
        };

        match unit_of_work.insert_store(&store).await {
            Ok(()) => {
                unit_of_work.commit().await?;
                Ok(if brand_was_created {
                    EnsureDefaultOrganizationOutcome::CreatedBrandAndStore {
                        store_id: store.id().as_str().to_string(),
                        brand_id: store.brand_id().as_str().to_string(),
                    }
                } else {
                    EnsureDefaultOrganizationOutcome::CreatedStore {
                        store_id: store.id().as_str().to_string(),
                        brand_id: store.brand_id().as_str().to_string(),
                    }
                })
            }
            Err(conflict @ ApplicationError::Conflict { .. }) => {
                unit_of_work.rollback().await?;
                self.recover_existing_store_or_error(
                    &input.brand_id,
                    &input.store_slug,
                    &input.store_name,
                    &input.store_currency_code,
                    &input.store_timezone,
                    conflict,
                )
                .await
            }
            Err(update_error) => {
                unit_of_work.rollback().await?;
                Err(update_error)
            }
        }
    }

    async fn recover_existing_store_or_error(
        &self,
        brand_id: &str,
        store_slug: &str,
        store_name: &str,
        store_currency_code: &str,
        store_timezone: &str,
        error: ApplicationError,
    ) -> Result<EnsureDefaultOrganizationOutcome, ApplicationError> {
        if let Some(outcome) = self.current_active_store_outcome().await? {
            return Ok(outcome);
        }

        let normalized_store_slug = Store::normalize_slug(store_slug)?;
        let mut unit_of_work = self.unit_of_work_factory.begin().await?;
        match self
            .resolve_exact_store_in_transaction(
                unit_of_work.as_mut(),
                &BrandId::new(brand_id),
                &normalized_store_slug,
                store_name,
                store_currency_code,
                store_timezone,
            )
            .await?
        {
            ExactStoreOutcome::Skipped(outcome) => {
                unit_of_work.rollback().await?;
                Ok(outcome)
            }
            ExactStoreOutcome::Recovered(outcome) => {
                unit_of_work.commit().await?;
                Ok(outcome)
            }
            ExactStoreOutcome::Missing => {
                unit_of_work.rollback().await?;
                Err(error)
            }
        }
    }

    async fn resolve_exact_store_in_transaction(
        &self,
        unit_of_work: &mut dyn OrganizationUnitOfWork,
        brand_id: &BrandId,
        normalized_store_slug: &str,
        store_name: &str,
        store_currency_code: &str,
        store_timezone: &str,
    ) -> Result<ExactStoreOutcome, ApplicationError> {
        let exact_store = unit_of_work
            .find_store_by_brand_slug(brand_id, normalized_store_slug)
            .await?;
        let Some(mut store) = exact_store else {
            return Ok(ExactStoreOutcome::Missing);
        };

        if store.status() == OrganizationStatus::Active && store.deleted_at().is_none() {
            return Ok(ExactStoreOutcome::Skipped(
                EnsureDefaultOrganizationOutcome::Skipped {
                    store_id: store.id().as_str().to_string(),
                    slug: store.slug().to_string(),
                },
            ));
        }

        store.restore_as_active(
            store_name,
            store_currency_code,
            store_timezone,
            self.clock.now(),
        )?;
        unit_of_work.update_store(&store).await?;
        Ok(ExactStoreOutcome::Recovered(
            EnsureDefaultOrganizationOutcome::RecoveredStore {
                store_id: store.id().as_str().to_string(),
                brand_id: store.brand_id().as_str().to_string(),
            },
        ))
    }
}

enum ExactStoreOutcome {
    Missing,
    Skipped(EnsureDefaultOrganizationOutcome),
    Recovered(EnsureDefaultOrganizationOutcome),
}

#[cfg(test)]
mod tests {
    use super::{
        EnsureDefaultOrganization, EnsureDefaultOrganizationInput, EnsureDefaultOrganizationOutcome,
    };
    use crate::{
        ApplicationError, Clock, IdGenerator, OrganizationUnitOfWork,
        OrganizationUnitOfWorkFactory, StoreQueryService, StoreReadRepository, StoreSummary,
    };
    use async_trait::async_trait;
    use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus, Store, StoreId};
    use ordering_food_shared_kernel::Timestamp;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

    const DEFAULT_BRAND_ID: &str = "00000000-0000-4000-8000-000000000001";

    #[derive(Default)]
    struct FakeUnitOfWorkFactory {
        state: Arc<Mutex<InMemoryOrganizationState>>,
        begin_count: Arc<Mutex<u32>>,
        commit_count: Arc<Mutex<u32>>,
        rollback_count: Arc<Mutex<u32>>,
    }

    #[async_trait]
    impl OrganizationUnitOfWorkFactory for FakeUnitOfWorkFactory {
        async fn begin(&self) -> Result<Box<dyn OrganizationUnitOfWork>, ApplicationError> {
            *self.begin_count.lock().unwrap() += 1;
            Ok(Box::new(InMemoryOrganizationUnitOfWork {
                state: self.state.clone(),
                commit_count: self.commit_count.clone(),
                rollback_count: self.rollback_count.clone(),
            }))
        }
    }

    struct InMemoryOrganizationUnitOfWork {
        state: Arc<Mutex<InMemoryOrganizationState>>,
        commit_count: Arc<Mutex<u32>>,
        rollback_count: Arc<Mutex<u32>>,
    }

    #[async_trait]
    impl OrganizationUnitOfWork for InMemoryOrganizationUnitOfWork {
        async fn find_brand_by_id(
            &mut self,
            brand_id: &BrandId,
        ) -> Result<Option<Brand>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .brands
                .get(brand_id.as_str())
                .cloned())
        }

        async fn insert_brand(&mut self, brand: &Brand) -> Result<(), ApplicationError> {
            let mut state = self.state.lock().unwrap();
            if state.fail_brand_insert_once {
                state.fail_brand_insert_once = false;
                if let Some(existing_brand) = state.brand_to_publish_on_conflict.take() {
                    state
                        .brands
                        .insert(existing_brand.id().as_str().to_string(), existing_brand);
                }
                return Err(ApplicationError::conflict("brand already exists"));
            }
            state
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
            let mut state = self.state.lock().unwrap();
            if state.fail_store_insert_once {
                state.fail_store_insert_once = false;
                if let Some(existing_store) = state.store_to_publish_on_conflict.take() {
                    state
                        .stores
                        .insert(existing_store.id().as_str().to_string(), existing_store);
                }
                return Err(ApplicationError::conflict("store already exists"));
            }
            state
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

    struct FakeClock;

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            datetime!(2026-04-05 08:00 UTC)
        }
    }

    struct FixedIdGenerator {
        next_store: Mutex<u32>,
    }

    impl Default for FixedIdGenerator {
        fn default() -> Self {
            Self {
                next_store: Mutex::new(1),
            }
        }
    }

    impl IdGenerator for FixedIdGenerator {
        fn next_brand_id(&self) -> BrandId {
            BrandId::new(DEFAULT_BRAND_ID)
        }

        fn next_store_id(&self) -> StoreId {
            let mut next = self.next_store.lock().unwrap();
            let value = format!("30000000-0000-4000-8000-{:012}", *next);
            *next += 1;
            StoreId::new(value)
        }
    }

    #[derive(Default)]
    struct InMemoryOrganizationState {
        brands: HashMap<String, Brand>,
        stores: HashMap<String, Store>,
        fail_brand_insert_once: bool,
        fail_store_insert_once: bool,
        brand_to_publish_on_conflict: Option<Brand>,
        store_to_publish_on_conflict: Option<Store>,
    }

    #[derive(Clone, Default)]
    struct InMemoryOrganizationRepository {
        state: Arc<Mutex<InMemoryOrganizationState>>,
    }

    #[async_trait]
    impl StoreReadRepository for InMemoryOrganizationRepository {
        async fn get_active(&self) -> Result<Option<StoreSummary>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .stores
                .values()
                .filter(|store| {
                    store.status() == OrganizationStatus::Active && store.deleted_at().is_none()
                })
                .min_by_key(|store| store.created_at())
                .map(|store| StoreSummary {
                    store_id: store.id().as_str().to_string(),
                    brand_id: store.brand_id().as_str().to_string(),
                    slug: store.slug().to_string(),
                    name: store.name().to_string(),
                    currency_code: store.currency_code().to_string(),
                    timezone: store.timezone().to_string(),
                    status: store.status().as_str().to_string(),
                }))
        }

        async fn get_by_id(
            &self,
            store_id: &StoreId,
        ) -> Result<Option<StoreSummary>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .stores
                .get(store_id.as_str())
                .map(|store| StoreSummary {
                    store_id: store.id().as_str().to_string(),
                    brand_id: store.brand_id().as_str().to_string(),
                    slug: store.slug().to_string(),
                    name: store.name().to_string(),
                    currency_code: store.currency_code().to_string(),
                    timezone: store.timezone().to_string(),
                    status: store.status().as_str().to_string(),
                }))
        }
    }

    fn build_use_case(
        repository: Arc<InMemoryOrganizationRepository>,
    ) -> (EnsureDefaultOrganization, Arc<FakeUnitOfWorkFactory>) {
        let unit_of_work_factory = Arc::new(FakeUnitOfWorkFactory {
            state: repository.state.clone(),
            ..Default::default()
        });
        let clock = Arc::new(FakeClock);
        let id_generator = Arc::new(FixedIdGenerator::default());
        let store_queries = Arc::new(StoreQueryService::new(repository.clone()));

        (
            EnsureDefaultOrganization::new(
                unit_of_work_factory.clone(),
                store_queries,
                clock,
                id_generator,
            ),
            unit_of_work_factory,
        )
    }

    fn default_input() -> EnsureDefaultOrganizationInput {
        EnsureDefaultOrganizationInput {
            brand_id: DEFAULT_BRAND_ID.to_string(),
            brand_slug: "ordering-food".to_string(),
            brand_name: "Ordering Food".to_string(),
            brand_status: "active".to_string(),
            store_slug: "ordering-food-demo".to_string(),
            store_name: "Ordering Food Demo Kitchen".to_string(),
            store_currency_code: "CNY".to_string(),
            store_timezone: "Asia/Shanghai".to_string(),
            store_status: "active".to_string(),
        }
    }

    #[tokio::test]
    async fn ensure_default_organization_creates_default_brand_and_store() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        let (use_case, _) = build_use_case(repository.clone());

        use_case.execute(default_input()).await.unwrap();

        let state = repository.state.lock().unwrap();
        assert_eq!(state.brands.len(), 1);
        assert_eq!(state.stores.len(), 1);
        assert!(state.brands.contains_key(DEFAULT_BRAND_ID));
        assert!(
            state
                .stores
                .values()
                .any(|store| store.slug() == "ordering-food-demo")
        );
    }

    #[tokio::test]
    async fn ensure_default_organization_skips_when_active_store_exists() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        repository.state.lock().unwrap().brands.insert(
            DEFAULT_BRAND_ID.to_string(),
            Brand::create(
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food",
                "Ordering Food",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:00 UTC),
            )
            .unwrap(),
        );
        let existing_store = Store::create(
            StoreId::new("store-existing"),
            BrandId::new(DEFAULT_BRAND_ID),
            "existing-store",
            "Existing Store",
            "CNY",
            "Asia/Shanghai",
            OrganizationStatus::Active,
            datetime!(2026-04-05 09:00 UTC),
        )
        .unwrap();
        repository
            .state
            .lock()
            .unwrap()
            .stores
            .insert(existing_store.id().as_str().to_string(), existing_store);
        let (use_case, _) = build_use_case(repository.clone());

        use_case.execute(default_input()).await.unwrap();

        let state = repository.state.lock().unwrap();
        assert_eq!(state.brands.len(), 1);
        assert_eq!(state.stores.len(), 1);
    }

    #[tokio::test]
    async fn ensure_default_organization_recovers_from_concurrent_store_conflict() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        repository.state.lock().unwrap().brands.insert(
            DEFAULT_BRAND_ID.to_string(),
            Brand::create(
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food",
                "Ordering Food",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:00 UTC),
            )
            .unwrap(),
        );
        repository.state.lock().unwrap().fail_store_insert_once = true;
        repository
            .state
            .lock()
            .unwrap()
            .store_to_publish_on_conflict = Some(
            Store::create(
                StoreId::new("store-concurrent"),
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food-demo",
                "Ordering Food Demo Kitchen",
                "CNY",
                "Asia/Shanghai",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:05 UTC),
            )
            .unwrap(),
        );
        let (use_case, _) = build_use_case(repository.clone());

        let outcome = use_case.execute(default_input()).await.unwrap();

        assert!(matches!(
            outcome,
            EnsureDefaultOrganizationOutcome::Skipped { store_id, .. }
            if store_id == "store-concurrent"
        ));
    }

    #[tokio::test]
    async fn ensure_default_organization_recovers_from_concurrent_brand_creation() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        repository.state.lock().unwrap().fail_brand_insert_once = true;
        repository
            .state
            .lock()
            .unwrap()
            .brand_to_publish_on_conflict = Some(
            Brand::create(
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food",
                "Ordering Food",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:00 UTC),
            )
            .unwrap(),
        );
        let (use_case, _) = build_use_case(repository.clone());

        let outcome = use_case.execute(default_input()).await.unwrap();

        assert!(matches!(
            outcome,
            EnsureDefaultOrganizationOutcome::CreatedBrandAndStore { brand_id, .. }
            if brand_id == DEFAULT_BRAND_ID
        ));
        assert_eq!(repository.state.lock().unwrap().stores.len(), 1);
    }

    #[tokio::test]
    async fn ensure_default_organization_reactivates_inactive_seed_store_after_conflict() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        repository.state.lock().unwrap().brands.insert(
            DEFAULT_BRAND_ID.to_string(),
            Brand::create(
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food",
                "Ordering Food",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:00 UTC),
            )
            .unwrap(),
        );
        repository.state.lock().unwrap().fail_store_insert_once = true;
        repository
            .state
            .lock()
            .unwrap()
            .store_to_publish_on_conflict = Some(
            Store::rehydrate(
                StoreId::new("store-inactive"),
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food-demo",
                "Ordering Food Demo Kitchen",
                "CNY",
                "Asia/Shanghai",
                OrganizationStatus::Inactive,
                datetime!(2026-04-05 08:00 UTC),
                datetime!(2026-04-05 08:00 UTC),
                None,
            )
            .unwrap(),
        );
        let (use_case, _) = build_use_case(repository.clone());

        let outcome = use_case.execute(default_input()).await.unwrap();
        let recovered_store = repository
            .state
            .lock()
            .unwrap()
            .stores
            .get("store-inactive")
            .cloned()
            .unwrap();

        assert!(matches!(
            outcome,
            EnsureDefaultOrganizationOutcome::RecoveredStore { store_id, .. }
            if store_id == "store-inactive"
        ));
        assert_eq!(recovered_store.status(), OrganizationStatus::Active);
        assert_eq!(recovered_store.deleted_at(), None);
    }

    #[tokio::test]
    async fn ensure_default_organization_restores_soft_deleted_seed_store_after_conflict() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        repository.state.lock().unwrap().brands.insert(
            DEFAULT_BRAND_ID.to_string(),
            Brand::create(
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food",
                "Ordering Food",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:00 UTC),
            )
            .unwrap(),
        );
        repository.state.lock().unwrap().fail_store_insert_once = true;
        repository
            .state
            .lock()
            .unwrap()
            .store_to_publish_on_conflict = Some(
            Store::rehydrate(
                StoreId::new("store-deleted"),
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food-demo",
                "Ordering Food Demo Kitchen",
                "CNY",
                "Asia/Shanghai",
                OrganizationStatus::Inactive,
                datetime!(2026-04-05 08:00 UTC),
                datetime!(2026-04-05 08:00 UTC),
                Some(datetime!(2026-04-05 08:30 UTC)),
            )
            .unwrap(),
        );
        let (use_case, _) = build_use_case(repository.clone());

        let outcome = use_case.execute(default_input()).await.unwrap();
        let recovered_store = repository
            .state
            .lock()
            .unwrap()
            .stores
            .get("store-deleted")
            .cloned()
            .unwrap();

        assert!(matches!(
            outcome,
            EnsureDefaultOrganizationOutcome::RecoveredStore { store_id, .. }
            if store_id == "store-deleted"
        ));
        assert_eq!(recovered_store.status(), OrganizationStatus::Active);
        assert_eq!(recovered_store.deleted_at(), None);
    }

    #[tokio::test]
    async fn ensure_default_organization_recovers_seed_store_with_non_normalized_slug_input() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        repository.state.lock().unwrap().brands.insert(
            DEFAULT_BRAND_ID.to_string(),
            Brand::create(
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food",
                "Ordering Food",
                OrganizationStatus::Active,
                datetime!(2026-04-05 08:00 UTC),
            )
            .unwrap(),
        );
        repository.state.lock().unwrap().fail_store_insert_once = true;
        repository
            .state
            .lock()
            .unwrap()
            .store_to_publish_on_conflict = Some(
            Store::rehydrate(
                StoreId::new("store-normalized"),
                BrandId::new(DEFAULT_BRAND_ID),
                "ordering-food-demo",
                "Ordering Food Demo Kitchen",
                "CNY",
                "Asia/Shanghai",
                OrganizationStatus::Inactive,
                datetime!(2026-04-05 08:00 UTC),
                datetime!(2026-04-05 08:00 UTC),
                None,
            )
            .unwrap(),
        );
        let (use_case, _) = build_use_case(repository.clone());
        let mut input = default_input();
        input.store_slug = " Ordering-Food-Demo ".to_string();

        let outcome = use_case.execute(input).await.unwrap();

        assert!(matches!(
            outcome,
            EnsureDefaultOrganizationOutcome::RecoveredStore { store_id, .. }
            if store_id == "store-normalized"
        ));
    }

    #[tokio::test]
    async fn ensure_default_organization_happy_path_runs_in_single_transaction() {
        let repository = Arc::new(InMemoryOrganizationRepository::default());
        let (use_case, transactions) = build_use_case(repository);

        let outcome = use_case.execute(default_input()).await.unwrap();

        assert!(matches!(
            outcome,
            EnsureDefaultOrganizationOutcome::CreatedBrandAndStore { .. }
        ));
        assert_eq!(*transactions.begin_count.lock().unwrap(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 0);
    }
}
