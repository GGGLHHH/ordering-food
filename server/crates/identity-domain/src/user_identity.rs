use crate::{IdentityType, NormalizedIdentifier};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserIdentity {
    identity_type: IdentityType,
    identifier_normalized: NormalizedIdentifier,
    bound_at: Timestamp,
}

impl UserIdentity {
    pub fn new(
        identity_type: IdentityType,
        identifier_normalized: NormalizedIdentifier,
        bound_at: Timestamp,
    ) -> Self {
        Self {
            identity_type,
            identifier_normalized,
            bound_at,
        }
    }

    pub fn identity_type(&self) -> &IdentityType {
        &self.identity_type
    }

    pub fn identifier_normalized(&self) -> &NormalizedIdentifier {
        &self.identifier_normalized
    }

    pub fn bound_at(&self) -> Timestamp {
        self.bound_at
    }
}
