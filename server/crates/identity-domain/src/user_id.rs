use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Identifier for UserId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for UserId {}

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
