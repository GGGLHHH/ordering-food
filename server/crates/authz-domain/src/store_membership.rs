use crate::StoreRole;
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMembership {
    user_id: String,
    store_id: String,
    role: StoreRole,
    granted_at: Timestamp,
}

impl StoreMembership {
    pub fn new(
        user_id: impl Into<String>,
        store_id: impl Into<String>,
        role: StoreRole,
        granted_at: Timestamp,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            store_id: store_id.into(),
            role,
            granted_at,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn store_id(&self) -> &str {
        &self.store_id
    }

    pub fn role(&self) -> StoreRole {
        self.role
    }

    pub fn granted_at(&self) -> Timestamp {
        self.granted_at
    }
}
