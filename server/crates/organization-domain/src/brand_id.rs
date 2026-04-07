use ordering_food_shared_kernel::{AggregateId, Identifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrandId(String);

impl BrandId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Identifier for BrandId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AggregateId for BrandId {}

impl From<String> for BrandId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
