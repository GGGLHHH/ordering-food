use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(String);

impl ItemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Identifier for ItemId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for ItemId {}

impl From<String> for ItemId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
