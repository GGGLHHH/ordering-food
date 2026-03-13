use crate::{
    ApplicationError, CategoryRepository, Clock, IdGenerator, StoreRepository, TransactionManager,
};
use ordering_food_menu_domain::{Category, MenuStatus, StoreId};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCategoryInput {
    pub store_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: String,
}

pub struct CreateCategory {
    store_repository: Arc<dyn StoreRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateCategory {
    pub fn new(
        store_repository: Arc<dyn StoreRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            store_repository,
            category_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateCategoryInput) -> Result<Category, ApplicationError> {
        let now = self.clock.now();
        let store_id = StoreId::new(input.store_id);
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

        let category = Category::create(
            self.id_generator.next_category_id(),
            store_id,
            input.slug,
            input.name,
            input.description,
            input.sort_order,
            MenuStatus::parse(input.status)?,
            now,
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
        Ok(category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApplicationError, CategoryRepository, Clock, IdGenerator, StoreRepository,
        TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_menu_domain::{CategoryId, ItemId, MenuStatus, Store, StoreId};
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
    struct FakeTransactionManager {
        rollback_count: Mutex<u32>,
    }

    #[async_trait]
    impl TransactionManager for FakeTransactionManager {
        async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
            Ok(Box::new(FakeTransactionContext))
        }

        async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            *self.rollback_count.lock().unwrap() += 1;
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
        categories: Mutex<Vec<Category>>,
    }

    #[async_trait]
    impl CategoryRepository for FakeCategoryRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _category_id: &ordering_food_menu_domain::CategoryId,
        ) -> Result<Option<Category>, ApplicationError> {
            Ok(None)
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            category: &Category,
        ) -> Result<(), ApplicationError> {
            self.categories.lock().unwrap().push(category.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_category_rejects_missing_store() {
        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = CreateCategory::new(
            Arc::new(FakeStoreRepository::default()),
            Arc::new(FakeCategoryRepository::default()),
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-13 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        );

        let error = use_case
            .execute(CreateCategoryInput {
                store_id: "missing-store".to_string(),
                slug: "main".to_string(),
                name: "Main".to_string(),
                description: None,
                sort_order: 0,
                status: MenuStatus::Active.as_str().to_string(),
            })
            .await
            .unwrap_err();

        assert!(
            matches!(error, ApplicationError::NotFound { ref message } if message == "store was not found")
        );
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 1);
    }
}
