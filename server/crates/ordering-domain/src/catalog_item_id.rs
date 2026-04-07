use ordering_food_shared_kernel::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogItemId(String);

impl CatalogItemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Identifier for CatalogItemId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CatalogItemId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
