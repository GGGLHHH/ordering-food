use ordering_food_shared_kernel::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoreId(String);

impl StoreId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Identifier for StoreId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for StoreId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
