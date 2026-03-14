use ordering_food_shared_kernel::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuItemId(String);

impl MenuItemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Identifier for MenuItemId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MenuItemId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
