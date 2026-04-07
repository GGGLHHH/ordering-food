use crate::AccessRoleRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMembershipRef {
    subject_id: String,
    store_id: String,
    role: AccessRoleRef,
}

impl StoreMembershipRef {
    pub fn new(
        subject_id: impl Into<String>,
        store_id: impl Into<String>,
        role: AccessRoleRef,
    ) -> Self {
        Self {
            subject_id: subject_id.into(),
            store_id: store_id.into(),
            role,
        }
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    pub fn store_id(&self) -> &str {
        &self.store_id
    }

    pub fn role(&self) -> AccessRoleRef {
        self.role
    }
}
