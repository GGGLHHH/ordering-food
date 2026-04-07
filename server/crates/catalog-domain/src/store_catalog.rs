use crate::{BrandId, DisplayRule, DomainError, SellableStatus, StoreCatalogId, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreCatalog {
    id: StoreCatalogId,
    brand_id: BrandId,
    store_id: StoreId,
    status: SellableStatus,
    display_rule: DisplayRule,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl StoreCatalog {
    pub fn attach(
        id: StoreCatalogId,
        brand_id: BrandId,
        store_id: StoreId,
        status: SellableStatus,
        display_rule: DisplayRule,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            brand_id,
            store_id,
            status,
            display_rule,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn id(&self) -> &StoreCatalogId {
        &self.id
    }

    pub fn brand_id(&self) -> &BrandId {
        &self.brand_id
    }

    pub fn store_id(&self) -> &StoreId {
        &self.store_id
    }

    pub fn status(&self) -> SellableStatus {
        self.status
    }

    pub fn display_rule(&self) -> DisplayRule {
        self.display_rule
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::StoreCatalog;
    use crate::{BrandId, DisplayRule, SellableStatus, StoreCatalogId, StoreId};
    use time::macros::datetime;

    #[test]
    fn store_catalog_requires_external_brand_and_store_scope() {
        let now = datetime!(2026-04-05 10:00 UTC);
        let catalog = StoreCatalog::attach(
            StoreCatalogId::new("store-catalog-1"),
            BrandId::new("brand-1"),
            StoreId::new("store-1"),
            SellableStatus::Sellable,
            DisplayRule::listed(),
            now,
        )
        .unwrap();

        assert_eq!(catalog.brand_id().as_str(), "brand-1");
        assert_eq!(catalog.store_id().as_str(), "store-1");
    }
}
