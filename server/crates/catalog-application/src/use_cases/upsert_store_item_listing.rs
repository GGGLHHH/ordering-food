use crate::{
    ApplicationError, Clock, ItemRepository, StoreCatalogRepository, StoreItemListingRepository,
    TransactionManager,
};
use ordering_food_catalog_domain::{
    DisplayRule, ItemId, Price, SellableStatus, StoreCatalogId, StoreItemListing,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertStoreItemListingInput {
    pub store_catalog_id: String,
    pub item_id: String,
    pub price_amount: i64,
    pub status: SellableStatus,
    pub display_rule: DisplayRule,
}

pub struct UpsertStoreItemListing {
    store_catalog_repository: Arc<dyn StoreCatalogRepository>,
    item_repository: Arc<dyn ItemRepository>,
    store_item_listing_repository: Arc<dyn StoreItemListingRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
}

impl UpsertStoreItemListing {
    pub fn new(
        store_catalog_repository: Arc<dyn StoreCatalogRepository>,
        item_repository: Arc<dyn ItemRepository>,
        store_item_listing_repository: Arc<dyn StoreItemListingRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            store_catalog_repository,
            item_repository,
            store_item_listing_repository,
            transaction_manager,
            clock,
        }
    }

    pub async fn execute(
        &self,
        input: UpsertStoreItemListingInput,
    ) -> Result<StoreItemListing, ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let store_catalog_id = StoreCatalogId::new(input.store_catalog_id);
        let item_id = ItemId::new(input.item_id);

        if self
            .store_catalog_repository
            .find_by_id(tx.as_mut(), &store_catalog_id)
            .await?
            .is_none()
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("store catalog was not found"));
        }

        if self
            .item_repository
            .find_by_id(tx.as_mut(), &item_id)
            .await?
            .is_none()
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("catalog item was not found"));
        }

        let listing = StoreItemListing::upsert(
            store_catalog_id,
            item_id,
            Price::new(input.price_amount)?,
            input.status,
            input.display_rule,
            self.clock.now(),
        );

        if let Err(error) = self
            .store_item_listing_repository
            .upsert(tx.as_mut(), &listing)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(listing)
    }
}

#[cfg(test)]
mod tests {
    use super::{UpsertStoreItemListing, UpsertStoreItemListingInput};
    use crate::{
        ApplicationError, Clock, ItemRepository, StoreCatalogRepository,
        StoreItemListingRepository, TransactionContext, TransactionManager,
    };
    use async_trait::async_trait;
    use ordering_food_catalog_domain::{
        BrandCatalogId, BrandId, CategoryId, DisplayRule, Item, ItemId, SellableStatus,
        StoreCatalog, StoreCatalogId, StoreId, StoreItemListing,
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

    #[derive(Default)]
    struct FakeStoreCatalogRepository {
        by_id: Mutex<HashMap<String, StoreCatalog>>,
    }

    #[async_trait]
    impl StoreCatalogRepository for FakeStoreCatalogRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            store_catalog_id: &StoreCatalogId,
        ) -> Result<Option<StoreCatalog>, ApplicationError> {
            Ok(self
                .by_id
                .lock()
                .unwrap()
                .get(store_catalog_id.as_str())
                .cloned())
        }

        async fn find_by_store_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _store_id: &StoreId,
        ) -> Result<Option<StoreCatalog>, ApplicationError> {
            Ok(None)
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            _store_catalog: &StoreCatalog,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeItemRepository {
        by_id: Mutex<HashMap<String, Item>>,
    }

    #[async_trait]
    impl ItemRepository for FakeItemRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            item_id: &ItemId,
        ) -> Result<Option<Item>, ApplicationError> {
            Ok(self.by_id.lock().unwrap().get(item_id.as_str()).cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            _item: &Item,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeStoreItemListingRepository {
        upserted: Mutex<Vec<StoreItemListing>>,
    }

    #[async_trait]
    impl StoreItemListingRepository for FakeStoreItemListingRepository {
        async fn upsert(
            &self,
            _tx: &mut dyn TransactionContext,
            listing: &StoreItemListing,
        ) -> Result<(), ApplicationError> {
            self.upserted.lock().unwrap().push(listing.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn upsert_store_item_listing_tracks_store_specific_price_and_visibility() {
        let store_repository = Arc::new(FakeStoreCatalogRepository::default());
        store_repository.by_id.lock().unwrap().insert(
            "store-catalog-1".to_string(),
            StoreCatalog::attach(
                StoreCatalogId::new("store-catalog-1"),
                BrandId::new("brand-1"),
                StoreId::new("store-1"),
                SellableStatus::Sellable,
                DisplayRule::listed(),
                datetime!(2026-04-05 10:00 UTC),
            )
            .unwrap(),
        );
        let item_repository = Arc::new(FakeItemRepository::default());
        item_repository.by_id.lock().unwrap().insert(
            "item-1".to_string(),
            Item::create(
                ItemId::new("item-1"),
                BrandCatalogId::new("brand-catalog-1"),
                CategoryId::new("category-1"),
                "dish",
                "Dish",
                None,
                None,
                1,
                datetime!(2026-04-05 10:00 UTC),
            )
            .unwrap(),
        );
        let listing_repository = Arc::new(FakeStoreItemListingRepository::default());
        let use_case = UpsertStoreItemListing::new(
            store_repository,
            item_repository,
            listing_repository.clone(),
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-04-05 10:00 UTC),
            }),
        );

        let listing = use_case
            .execute(UpsertStoreItemListingInput {
                store_catalog_id: "store-catalog-1".to_string(),
                item_id: "item-1".to_string(),
                price_amount: 3200,
                status: SellableStatus::Sellable,
                display_rule: DisplayRule::listed(),
            })
            .await
            .unwrap();

        assert_eq!(listing.price().amount(), 3200);
        assert_eq!(listing_repository.upserted.lock().unwrap().len(), 1);
    }
}
