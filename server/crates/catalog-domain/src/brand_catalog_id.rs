use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandCatalogId(String);

impl BrandCatalogId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Identifier for BrandCatalogId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for BrandCatalogId {}

impl From<String> for BrandCatalogId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
