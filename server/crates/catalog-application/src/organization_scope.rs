#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogBrandScope {
    pub brand_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogStoreScope {
    pub store_id: String,
    pub brand_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
}
