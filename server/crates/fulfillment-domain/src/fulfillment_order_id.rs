#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FulfillmentOrderId(String);

impl FulfillmentOrderId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
