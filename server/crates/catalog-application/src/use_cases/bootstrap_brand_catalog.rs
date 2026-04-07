use crate::{
    ApplicationError, BrandCatalogRepository, Clock, IdGenerator, OrganizationScopeReader,
    TransactionManager,
};
use ordering_food_catalog_domain::{BrandCatalog, BrandId};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapBrandCatalogInput {
    pub brand_id: String,
    pub slug: String,
    pub name: String,
}

pub struct BootstrapBrandCatalog {
    organization_scope_reader: Arc<dyn OrganizationScopeReader>,
    brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl BootstrapBrandCatalog {
    pub fn new(
        organization_scope_reader: Arc<dyn OrganizationScopeReader>,
        brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            organization_scope_reader,
            brand_catalog_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(
        &self,
        input: BootstrapBrandCatalogInput,
    ) -> Result<BrandCatalog, ApplicationError> {
        let brand = self
            .organization_scope_reader
            .get_brand(&input.brand_id)
            .await?;
        if brand.is_none() {
            return Err(ApplicationError::not_found("brand scope was not found"));
        }

        let mut tx = self.transaction_manager.begin().await?;
        let brand_id = BrandId::new(input.brand_id);

        if self
            .brand_catalog_repository
            .find_by_brand_id(tx.as_mut(), &brand_id)
            .await?
            .is_some()
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::conflict(
                "brand catalog already exists for brand scope",
            ));
        }

        let brand_catalog = BrandCatalog::create(
            self.id_generator.next_brand_catalog_id(),
            brand_id,
            input.slug,
            input.name,
            self.clock.now(),
        )?;

        if let Err(error) = self
            .brand_catalog_repository
            .insert(tx.as_mut(), &brand_catalog)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(brand_catalog)
    }
}

#[cfg(test)]
mod tests {
    use super::{BootstrapBrandCatalog, BootstrapBrandCatalogInput};
    use crate::{
        ApplicationError, BrandCatalogRepository, Clock, IdGenerator, OrganizationScopeReader,
        TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_catalog_domain::{
        BrandCatalog, BrandCatalogId, BrandId, CategoryId, ItemId, StoreCatalogId,
    };
    use ordering_food_organization_published::{BrandRef, StoreSummary};
    use ordering_food_shared_kernel::Timestamp;
    use std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

    #[derive(Default)]
    struct FakeTransactionContext;

    impl TransactionContext for FakeTransactionContext {
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
            self
        }
    }

    #[derive(Default)]
    struct FakeTransactionManager;

    #[async_trait]
    impl TransactionManager for FakeTransactionManager {
        async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
            Ok(Box::new(FakeTransactionContext))
        }

        async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    struct FakeClock {
        now: Timestamp,
    }

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FakeIdGenerator;

    impl IdGenerator for FakeIdGenerator {
        fn next_brand_catalog_id(&self) -> BrandCatalogId {
            BrandCatalogId::new("brand-catalog-generated")
        }

        fn next_store_catalog_id(&self) -> StoreCatalogId {
            StoreCatalogId::new("store-catalog-generated")
        }

        fn next_category_id(&self) -> CategoryId {
            CategoryId::new("category-generated")
        }

        fn next_item_id(&self) -> ItemId {
            ItemId::new("item-generated")
        }
    }

    struct FakeOrganizationScopeReader {
        brand: Option<BrandRef>,
    }

    impl FakeOrganizationScopeReader {
        fn missing() -> Self {
            Self { brand: None }
        }

        fn existing() -> Self {
            Self {
                brand: Some(BrandRef {
                    brand_id: "brand-1".to_string(),
                }),
            }
        }
    }

    #[async_trait]
    impl OrganizationScopeReader for FakeOrganizationScopeReader {
        async fn get_active_store(&self) -> Result<Option<StoreSummary>, ApplicationError> {
            Ok(None)
        }

        async fn get_brand(&self, _brand_id: &str) -> Result<Option<BrandRef>, ApplicationError> {
            Ok(self.brand.clone())
        }

        async fn get_store_scope(
            &self,
            _brand_id: &str,
            _store_id: &str,
        ) -> Result<Option<StoreSummary>, ApplicationError> {
            Ok(None)
        }
    }

    #[derive(Default)]
    struct FakeBrandCatalogRepository {
        catalogs_by_brand: Mutex<HashMap<String, BrandCatalog>>,
        inserted: Mutex<Vec<BrandCatalog>>,
    }

    #[async_trait]
    impl BrandCatalogRepository for FakeBrandCatalogRepository {
        async fn find_by_brand_id(
            &self,
            _tx: &mut dyn TransactionContext,
            brand_id: &BrandId,
        ) -> Result<Option<BrandCatalog>, ApplicationError> {
            Ok(self
                .catalogs_by_brand
                .lock()
                .unwrap()
                .get(brand_id.as_str())
                .cloned())
        }

        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _brand_catalog_id: &BrandCatalogId,
        ) -> Result<Option<BrandCatalog>, ApplicationError> {
            Ok(None)
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            brand_catalog: &BrandCatalog,
        ) -> Result<(), ApplicationError> {
            self.inserted.lock().unwrap().push(brand_catalog.clone());
            self.catalogs_by_brand.lock().unwrap().insert(
                brand_catalog.brand_id().as_str().to_string(),
                brand_catalog.clone(),
            );
            Ok(())
        }
    }

    #[tokio::test]
    async fn bootstrap_brand_catalog_requires_existing_brand_scope() {
        let use_case = BootstrapBrandCatalog::new(
            Arc::new(FakeOrganizationScopeReader::missing()),
            Arc::new(FakeBrandCatalogRepository::default()),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let error = use_case
            .execute(BootstrapBrandCatalogInput {
                brand_id: "brand-1".to_string(),
                slug: "demo-catalog".to_string(),
                name: "Demo Catalog".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::NotFound { .. }));
    }

    #[tokio::test]
    async fn bootstrap_brand_catalog_persists_catalog_for_existing_brand_scope() {
        let repository = Arc::new(FakeBrandCatalogRepository::default());
        let use_case = BootstrapBrandCatalog::new(
            Arc::new(FakeOrganizationScopeReader::existing()),
            repository.clone(),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let brand_catalog = use_case
            .execute(BootstrapBrandCatalogInput {
                brand_id: "brand-1".to_string(),
                slug: "demo-catalog".to_string(),
                name: "Demo Catalog".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(brand_catalog.brand_id().as_str(), "brand-1");
        assert_eq!(repository.inserted.lock().unwrap().len(), 1);
    }
}
