use crate::{DisplayRule, ItemId, Price, SellableStatus, StoreCatalogId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreItemListing {
    store_catalog_id: StoreCatalogId,
    item_id: ItemId,
    price: Price,
    status: SellableStatus,
    display_rule: DisplayRule,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl StoreItemListing {
    pub fn upsert(
        store_catalog_id: StoreCatalogId,
        item_id: ItemId,
        price: Price,
        status: SellableStatus,
        display_rule: DisplayRule,
        now: Timestamp,
    ) -> Self {
        Self {
            store_catalog_id,
            item_id,
            price,
            status,
            display_rule,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn store_catalog_id(&self) -> &StoreCatalogId {
        &self.store_catalog_id
    }

    pub fn item_id(&self) -> &ItemId {
        &self.item_id
    }

    pub fn price(&self) -> Price {
        self.price
    }

    pub fn status(&self) -> SellableStatus {
        self.status
    }

    pub fn display_rule(&self) -> DisplayRule {
        self.display_rule
    }
}
