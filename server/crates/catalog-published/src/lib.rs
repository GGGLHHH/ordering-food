//! Published contracts for the Catalog bounded context.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogItemRef {
    pub item_id: String,
    pub brand_id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogPriceFact {
    pub item_id: String,
    pub store_id: String,
    pub price_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreCatalogRef {
    pub brand_id: String,
    pub store_id: String,
}
