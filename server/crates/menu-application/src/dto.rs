use ordering_food_menu_domain::CategoryId;
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreReadModel {
    pub store_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryReadModel {
    pub category_id: String,
    pub store_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemReadModel {
    pub item_id: String,
    pub store_id: String,
    pub category_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub price_amount: i64,
    pub sort_order: i32,
    pub status: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Default)]
pub struct ItemListFilter {
    pub category_id: Option<CategoryId>,
}
