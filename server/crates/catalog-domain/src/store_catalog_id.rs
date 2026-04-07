use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoreCatalogId(String);

impl StoreCatalogId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Identifier for StoreCatalogId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for StoreCatalogId {}

impl From<String> for StoreCatalogId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
