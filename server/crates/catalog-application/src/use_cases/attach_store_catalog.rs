use crate::{
    ApplicationError, Clock, IdGenerator, OrganizationScopeReader, StoreCatalogRepository,
    TransactionManager,
};
use ordering_food_catalog_domain::{BrandId, DisplayRule, SellableStatus, StoreCatalog, StoreId};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachStoreCatalogInput {
    pub brand_id: String,
    pub store_id: String,
}

pub struct AttachStoreCatalog {
    organization_scope_reader: Arc<dyn OrganizationScopeReader>,
    store_catalog_repository: Arc<dyn StoreCatalogRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl AttachStoreCatalog {
    pub fn new(
        organization_scope_reader: Arc<dyn OrganizationScopeReader>,
        store_catalog_repository: Arc<dyn StoreCatalogRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            organization_scope_reader,
            store_catalog_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(
        &self,
        input: AttachStoreCatalogInput,
    ) -> Result<String, ApplicationError> {
        let store_scope = self
            .organization_scope_reader
            .get_store_scope(&input.brand_id, &input.store_id)
            .await?;
        let Some(store_scope) = store_scope else {
            return Err(ApplicationError::not_found("store scope was not found"));
        };
        if store_scope.brand_id != input.brand_id {
            return Err(ApplicationError::not_found("store scope was not found"));
        }

        let mut tx = self.transaction_manager.begin().await?;
        let store_id = StoreId::new(input.store_id);

        if self
            .store_catalog_repository
            .find_by_store_id(tx.as_mut(), &store_id)
            .await?
            .is_some()
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::conflict(
                "store catalog already exists for store scope",
            ));
        }

        let store_catalog = StoreCatalog::attach(
            self.id_generator.next_store_catalog_id(),
            BrandId::new(input.brand_id),
            store_id,
            SellableStatus::Sellable,
            DisplayRule::listed(),
            self.clock.now(),
        )?;

        if let Err(error) = self
            .store_catalog_repository
            .insert(tx.as_mut(), &store_catalog)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(store_catalog.id().as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{AttachStoreCatalog, AttachStoreCatalogInput};
    use crate::{
        ApplicationError, Clock, IdGenerator, OrganizationScopeReader, StoreCatalogRepository,
        TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_catalog_domain::{
        CategoryId, ItemId, StoreCatalog, StoreCatalogId, StoreId,
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
        fn next_brand_catalog_id(&self) -> ordering_food_catalog_domain::BrandCatalogId {
            ordering_food_catalog_domain::BrandCatalogId::new("brand-catalog-generated")
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
        store_scope: Option<StoreSummary>,
    }

    impl FakeOrganizationScopeReader {
        fn missing() -> Self {
            Self { store_scope: None }
        }

        fn existing() -> Self {
            Self {
                store_scope: Some(StoreSummary {
                    brand_id: "brand-1".to_string(),
                    store_id: "store-1".to_string(),
                    slug: "demo-store".to_string(),
                    name: "Demo Store".to_string(),
                    currency_code: "CNY".to_string(),
                    timezone: "Asia/Shanghai".to_string(),
                    status: "active".to_string(),
                }),
            }
        }
    }

    #[async_trait]
    impl OrganizationScopeReader for FakeOrganizationScopeReader {
        async fn get_active_store(&self) -> Result<Option<StoreSummary>, ApplicationError> {
            Ok(self.store_scope.clone())
        }

        async fn get_brand(&self, _brand_id: &str) -> Result<Option<BrandRef>, ApplicationError> {
            Ok(None)
        }

        async fn get_store_scope(
            &self,
            _brand_id: &str,
            _store_id: &str,
        ) -> Result<Option<StoreSummary>, ApplicationError> {
            Ok(self.store_scope.clone())
        }
    }

    #[derive(Default)]
    struct FakeStoreCatalogRepository {
        by_store: Mutex<HashMap<String, StoreCatalog>>,
        inserted: Mutex<Vec<StoreCatalog>>,
    }

    #[async_trait]
    impl StoreCatalogRepository for FakeStoreCatalogRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            store_catalog_id: &StoreCatalogId,
        ) -> Result<Option<StoreCatalog>, ApplicationError> {
            Ok(self
                .inserted
                .lock()
                .unwrap()
                .iter()
                .find(|catalog| catalog.id() == store_catalog_id)
                .cloned())
        }

        async fn find_by_store_id(
            &self,
            _tx: &mut dyn TransactionContext,
            store_id: &StoreId,
        ) -> Result<Option<StoreCatalog>, ApplicationError> {
            Ok(self
                .by_store
                .lock()
                .unwrap()
                .get(store_id.as_str())
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            store_catalog: &StoreCatalog,
        ) -> Result<(), ApplicationError> {
            self.by_store.lock().unwrap().insert(
                store_catalog.store_id().as_str().to_string(),
                store_catalog.clone(),
            );
            self.inserted.lock().unwrap().push(store_catalog.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn attach_store_catalog_rejects_unknown_store_scope() {
        let use_case = AttachStoreCatalog::new(
            Arc::new(FakeOrganizationScopeReader::missing()),
            Arc::new(FakeStoreCatalogRepository::default()),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let error = use_case
            .execute(AttachStoreCatalogInput {
                brand_id: "brand-1".to_string(),
                store_id: "store-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::NotFound { .. }));
    }

    #[tokio::test]
    async fn attach_store_catalog_persists_store_scope_attachment() {
        let repository = Arc::new(FakeStoreCatalogRepository::default());
        let use_case = AttachStoreCatalog::new(
            Arc::new(FakeOrganizationScopeReader::existing()),
            repository.clone(),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let store_catalog_id = use_case
            .execute(AttachStoreCatalogInput {
                brand_id: "brand-1".to_string(),
                store_id: "store-1".to_string(),
            })
            .await
            .unwrap();

        assert!(!store_catalog_id.is_empty());
        assert_eq!(repository.inserted.lock().unwrap().len(), 1);
    }
}
