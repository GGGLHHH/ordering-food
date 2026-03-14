use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(String);

impl OrderId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Identifier for OrderId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for OrderId {}

impl From<String> for OrderId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
