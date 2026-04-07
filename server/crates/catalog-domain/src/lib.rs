mod brand_catalog;
mod brand_catalog_id;
mod brand_id;
mod category;
mod category_id;
mod display_rule;
mod error;
mod item;
mod item_id;
mod price;
mod sellable_status;
mod store_catalog;
mod store_catalog_id;
mod store_id;
mod store_item_listing;

pub use brand_catalog::BrandCatalog;
pub use brand_catalog_id::BrandCatalogId;
pub use brand_id::BrandId;
pub use category::Category;
pub use category_id::CategoryId;
pub use display_rule::DisplayRule;
pub use error::DomainError;
pub use item::Item;
pub use item_id::ItemId;
pub use price::Price;
pub use sellable_status::SellableStatus;
pub use store_catalog::StoreCatalog;
pub use store_catalog_id::StoreCatalogId;
pub use store_id::StoreId;
pub use store_item_listing::StoreItemListing;

use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogContextSnapshot {
    captured_at: Timestamp,
}

impl CatalogContextSnapshot {
    pub fn new(captured_at: Timestamp) -> Self {
        Self { captured_at }
    }

    pub fn captured_at(&self) -> Timestamp {
        self.captured_at
    }
}
