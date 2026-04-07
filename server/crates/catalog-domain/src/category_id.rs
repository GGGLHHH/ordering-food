use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryId(String);

impl CategoryId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Identifier for CategoryId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for CategoryId {}

impl From<String> for CategoryId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
