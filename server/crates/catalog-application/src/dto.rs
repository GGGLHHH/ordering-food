use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandCatalogReadModel {
    pub brand_catalog_id: String,
    pub brand_id: String,
    pub slug: String,
    pub name: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreCatalogReadModel {
    pub store_catalog_id: String,
    pub brand_id: String,
    pub store_id: String,
    pub status: String,
    pub display_rule: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryReadModel {
    pub category_id: String,
    pub brand_catalog_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemReadModel {
    pub item_id: String,
    pub brand_catalog_id: String,
    pub category_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreItemListingReadModel {
    pub store_catalog_id: String,
    pub item_id: String,
    pub price_amount: i64,
    pub status: String,
    pub display_rule: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
