#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccessScope {
    Platform,
    Store { store_id: String },
}

impl AccessScope {
    pub fn platform() -> Self {
        Self::Platform
    }

    pub fn store(store_id: impl Into<String>) -> Self {
        Self::Store {
            store_id: store_id.into(),
        }
    }

    pub fn is_platform(&self) -> bool {
        matches!(self, Self::Platform)
    }

    pub fn matches_store(&self, store_id: &str) -> bool {
        match self {
            Self::Platform => false,
            Self::Store {
                store_id: scoped_store_id,
            } => scoped_store_id == store_id,
        }
    }

    pub fn store_id(&self) -> Option<&str> {
        match self {
            Self::Platform => None,
            Self::Store { store_id } => Some(store_id),
        }
    }
}
