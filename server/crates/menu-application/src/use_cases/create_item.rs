use crate::{
    ApplicationError, CategoryRepository, Clock, IdGenerator, ItemRepository, StoreRepository,
    TransactionManager,
};
use ordering_food_menu_domain::{CategoryId, Item, MenuStatus, StoreId};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateItemInput {
    pub store_id: String,
    pub category_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub price_amount: i64,
    pub sort_order: i32,
    pub status: String,
}

pub struct CreateItem {
    store_repository: Arc<dyn StoreRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    item_repository: Arc<dyn ItemRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateItem {
    pub fn new(
        store_repository: Arc<dyn StoreRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        item_repository: Arc<dyn ItemRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            store_repository,
            category_repository,
            item_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateItemInput) -> Result<Item, ApplicationError> {
        let now = self.clock.now();
        let store_id = StoreId::new(input.store_id);
        let category_id = CategoryId::new(input.category_id);
        let mut tx = self.transaction_manager.begin().await?;

        let store = self
            .store_repository
            .find_by_id(tx.as_mut(), &store_id)
            .await?;
        let Some(store) = store else {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("store was not found"));
        };
        if store.is_deleted() {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("store was not found"));
        }

        let category = self
            .category_repository
            .find_by_id(tx.as_mut(), &category_id)
            .await?;
        let Some(category) = category else {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("category was not found"));
        };
        if category.is_deleted() {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("category was not found"));
        }
        if category.store_id() != &store_id {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::validation(
                "category does not belong to store",
            ));
        }

        let item = Item::create(
            self.id_generator.next_item_id(),
            store_id,
            category_id,
            input.slug,
            input.name,
            input.description,
            input.image_url,
            input.price_amount,
            input.sort_order,
            MenuStatus::parse(input.status)?,
            now,
        )?;

        if let Err(error) = self.item_repository.insert(tx.as_mut(), &item).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApplicationError, CategoryRepository, Clock, IdGenerator, ItemRepository, StoreRepository,
        TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_menu_domain::{Category, CategoryId, ItemId, MenuStatus, Store, StoreId};
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
        fn next_store_id(&self) -> StoreId {
            StoreId::new("store-generated")
        }

        fn next_category_id(&self) -> CategoryId {
            CategoryId::new("category-generated")
        }

        fn next_item_id(&self) -> ItemId {
            ItemId::new("item-generated")
        }
    }

    #[derive(Default)]
    struct FakeStoreRepository {
        stores: Mutex<HashMap<String, Store>>,
    }

    #[async_trait]
    impl StoreRepository for FakeStoreRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            store_id: &StoreId,
        ) -> Result<Option<Store>, ApplicationError> {
            Ok(self.stores.lock().unwrap().get(store_id.as_str()).cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            _store: &Store,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeCategoryRepository {
        categories: Mutex<HashMap<String, Category>>,
    }

    #[async_trait]
    impl CategoryRepository for FakeCategoryRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            category_id: &CategoryId,
        ) -> Result<Option<Category>, ApplicationError> {
            Ok(self
                .categories
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
        items: Mutex<Vec<Item>>,
    }

    #[async_trait]
    impl ItemRepository for FakeItemRepository {
        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            item: &Item,
        ) -> Result<(), ApplicationError> {
            self.items.lock().unwrap().push(item.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_item_rejects_category_from_another_store() {
        let now = datetime!(2026-03-13 10:00 UTC);
        let store = Store::create(
            StoreId::new("store-1"),
            "store-1",
            "Store 1",
            "CNY",
            "Asia/Shanghai",
            MenuStatus::Active,
            now,
        )
        .unwrap();
        let category = Category::create(
            CategoryId::new("category-1"),
            StoreId::new("store-2"),
            "mains",
            "Mains",
            None,
            0,
            MenuStatus::Active,
            now,
        )
        .unwrap();
        let store_repository = Arc::new(FakeStoreRepository::default());
        store_repository
            .stores
            .lock()
            .unwrap()
            .insert(store.id().as_str().to_string(), store);
        let category_repository = Arc::new(FakeCategoryRepository::default());
        category_repository
            .categories
            .lock()
            .unwrap()
            .insert(category.id().as_str().to_string(), category);

        let use_case = CreateItem::new(
            store_repository,
            category_repository,
            Arc::new(FakeItemRepository::default()),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock { now }),
            Arc::new(FakeIdGenerator),
        );

        let error = use_case
            .execute(CreateItemInput {
                store_id: "store-1".to_string(),
                category_id: "category-1".to_string(),
                slug: "rice".to_string(),
                name: "Rice".to_string(),
                description: None,
                image_url: None,
                price_amount: 1200,
                sort_order: 0,
                status: MenuStatus::Active.as_str().to_string(),
            })
            .await
            .unwrap_err();

        assert!(
            matches!(error, ApplicationError::Validation { ref message } if message == "category does not belong to store")
        );
    }
}
