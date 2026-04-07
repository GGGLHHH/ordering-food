use crate::{
    ApplicationError, BrandCatalogRepository, CategoryRepository, Clock, IdGenerator,
    TransactionManager,
};
use ordering_food_catalog_domain::{BrandCatalogId, Category};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCategoryInput {
    pub brand_catalog_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

pub struct CreateCategory {
    brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateCategory {
    pub fn new(
        brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            brand_catalog_repository,
            category_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateCategoryInput) -> Result<String, ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let brand_catalog_id = BrandCatalogId::new(input.brand_catalog_id);

        let brand_catalog = self
            .brand_catalog_repository
            .find_by_id(tx.as_mut(), &brand_catalog_id)
            .await?;
        if brand_catalog.is_none() {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("brand catalog was not found"));
        }

        let category = Category::create(
            self.id_generator.next_category_id(),
            brand_catalog_id,
            input.slug,
            input.name,
            input.description,
            input.sort_order,
            self.clock.now(),
        )?;

        if let Err(error) = self
            .category_repository
            .insert(tx.as_mut(), &category)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(category.id().as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateCategory, CreateCategoryInput};
    use crate::{
        ApplicationError, BrandCatalogRepository, CategoryRepository, Clock, IdGenerator,
        TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_catalog_domain::{
        BrandCatalog, BrandCatalogId, BrandId, Category, CategoryId, ItemId, StoreCatalogId,
    };
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

    #[derive(Default)]
    struct FakeBrandCatalogRepository {
        by_id: Mutex<HashMap<String, BrandCatalog>>,
    }

    #[async_trait]
    impl BrandCatalogRepository for FakeBrandCatalogRepository {
        async fn find_by_brand_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _brand_id: &BrandId,
        ) -> Result<Option<BrandCatalog>, ApplicationError> {
            Ok(None)
        }

        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            brand_catalog_id: &BrandCatalogId,
        ) -> Result<Option<BrandCatalog>, ApplicationError> {
            Ok(self
                .by_id
                .lock()
                .unwrap()
                .get(brand_catalog_id.as_str())
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            _brand_catalog: &BrandCatalog,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeCategoryRepository {
        inserted: Mutex<Vec<Category>>,
    }

    #[async_trait]
    impl CategoryRepository for FakeCategoryRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _category_id: &CategoryId,
        ) -> Result<Option<Category>, ApplicationError> {
            Ok(None)
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            category: &Category,
        ) -> Result<(), ApplicationError> {
            self.inserted.lock().unwrap().push(category.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_category_requires_existing_brand_catalog() {
        let brand_catalog_repository = Arc::new(FakeBrandCatalogRepository::default());
        brand_catalog_repository.by_id.lock().unwrap().insert(
            "brand-catalog-1".to_string(),
            BrandCatalog::create(
                BrandCatalogId::new("brand-catalog-1"),
                BrandId::new("brand-1"),
                "demo-catalog",
                "Demo Catalog",
                datetime!(2026-04-05 10:00 UTC),
            )
            .unwrap(),
        );
        let category_repository = Arc::new(FakeCategoryRepository::default());
        let use_case = CreateCategory::new(
            brand_catalog_repository,
            category_repository.clone(),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let category_id = use_case
            .execute(CreateCategoryInput {
                brand_catalog_id: "brand-catalog-1".to_string(),
                slug: "featured".to_string(),
                name: "Featured".to_string(),
                description: None,
                sort_order: 1,
            })
            .await
            .unwrap();

        assert!(!category_id.is_empty());
        assert_eq!(category_repository.inserted.lock().unwrap().len(), 1);
    }
}
