use ordering_food_shared_kernel::Timestamp;

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

pub trait UuidGenerator: Send + Sync {
    fn next_uuid(&self) -> uuid::Uuid;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
