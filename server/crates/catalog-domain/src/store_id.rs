use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoreId(String);

impl StoreId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Identifier for StoreId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for StoreId {}

impl From<String> for StoreId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
