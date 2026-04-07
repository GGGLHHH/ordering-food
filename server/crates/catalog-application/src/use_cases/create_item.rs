use crate::{
    ApplicationError, BrandCatalogRepository, CategoryRepository, Clock, IdGenerator,
    ItemRepository, TransactionManager,
};
use ordering_food_catalog_domain::{BrandCatalogId, CategoryId, Item};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateItemInput {
    pub brand_catalog_id: String,
    pub category_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
}

pub struct CreateItem {
    brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    item_repository: Arc<dyn ItemRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateItem {
    pub fn new(
        brand_catalog_repository: Arc<dyn BrandCatalogRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        item_repository: Arc<dyn ItemRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            brand_catalog_repository,
            category_repository,
            item_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateItemInput) -> Result<String, ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let brand_catalog_id = BrandCatalogId::new(input.brand_catalog_id);
        let category_id = CategoryId::new(input.category_id);

        let brand_catalog = self
            .brand_catalog_repository
            .find_by_id(tx.as_mut(), &brand_catalog_id)
            .await?;
        if brand_catalog.is_none() {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("brand catalog was not found"));
        }

        let category = self
            .category_repository
            .find_by_id(tx.as_mut(), &category_id)
            .await?;
        let Some(category) = category else {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("category was not found"));
        };
        if category.brand_catalog_id() != &brand_catalog_id {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::validation(
                "category does not belong to brand catalog",
            ));
        }

        let item = Item::create(
            self.id_generator.next_item_id(),
            brand_catalog_id,
            category_id,
            input.slug,
            input.name,
            input.description,
            input.image_url,
            input.sort_order,
            self.clock.now(),
        )?;

        if let Err(error) = self.item_repository.insert(tx.as_mut(), &item).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(item.id().as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateItem, CreateItemInput};
    use crate::{
        ApplicationError, BrandCatalogRepository, CategoryRepository, Clock, IdGenerator,
        ItemRepository, TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_catalog_domain::{
        BrandCatalog, BrandCatalogId, BrandId, Category, CategoryId, Item, ItemId, StoreCatalogId,
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
        by_id: Mutex<HashMap<String, Category>>,
    }

    #[async_trait]
    impl CategoryRepository for FakeCategoryRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            category_id: &CategoryId,
        ) -> Result<Option<Category>, ApplicationError> {
            Ok(self
                .by_id
                .lock()
                .unwrap()
                .get(category_id.as_str())
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            _category: &Category,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeItemRepository {
        inserted: Mutex<Vec<Item>>,
    }

    #[async_trait]
    impl ItemRepository for FakeItemRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _item_id: &ItemId,
        ) -> Result<Option<Item>, ApplicationError> {
            Ok(None)
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            item: &Item,
        ) -> Result<(), ApplicationError> {
            self.inserted.lock().unwrap().push(item.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_item_rejects_category_from_other_brand_catalog() {
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
        category_repository.by_id.lock().unwrap().insert(
            "category-1".to_string(),
            Category::create(
                CategoryId::new("category-1"),
                BrandCatalogId::new("other-brand-catalog"),
                "featured",
                "Featured",
                None,
                1,
                datetime!(2026-04-05 10:00 UTC),
            )
            .unwrap(),
        );
        let use_case = CreateItem::new(
            brand_catalog_repository,
            category_repository,
            Arc::new(FakeItemRepository::default()),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let error = use_case
            .execute(CreateItemInput {
                brand_catalog_id: "brand-catalog-1".to_string(),
                category_id: "category-1".to_string(),
                slug: "dish".to_string(),
                name: "Dish".to_string(),
                description: None,
                image_url: None,
                sort_order: 1,
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Validation { .. }));
    }
}
