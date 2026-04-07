#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandRef {
    pub brand_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRef {
    pub store_id: String,
    pub brand_id: String,
}
