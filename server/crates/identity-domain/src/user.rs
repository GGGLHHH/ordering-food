use crate::{DomainError, UserId, UserIdentity, UserProfile, UserStatus};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    status: UserStatus,
    profile: UserProfile,
    identities: Vec<UserIdentity>,
    created_at: Timestamp,
    updated_at: Timestamp,
    deleted_at: Option<Timestamp>,
}

impl User {
    pub fn create(id: UserId, profile: UserProfile, now: Timestamp) -> Self {
        Self {
            id,
            status: UserStatus::Active,
            profile,
            identities: Vec::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn rehydrate(
        id: UserId,
        status: UserStatus,
        profile: UserProfile,
        identities: Vec<UserIdentity>,
        created_at: Timestamp,
        updated_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        let mut user = Self {
            id,
            status,
            profile,
            identities: Vec::new(),
            created_at,
            updated_at,
            deleted_at,
        };

        if user.deleted_at.is_some() && user.status != UserStatus::Disabled {
            return Err(DomainError::DeletedUserMustBeDisabled);
        }

        for identity in identities {
            if user.has_identity(&identity) {
                return Err(DomainError::DuplicateIdentity);
            }
            user.identities.push(identity);
        }

        Ok(user)
    }

    pub fn update_profile(
        &mut self,
        profile: UserProfile,
        now: Timestamp,
    ) -> Result<(), DomainError> {
        self.ensure_not_deleted()?;
        self.profile = profile;
        self.updated_at = now;
        Ok(())
    }

    pub fn bind_identity(
        &mut self,
        identity: UserIdentity,
        now: Timestamp,
    ) -> Result<(), DomainError> {
        self.ensure_not_deleted()?;
        if self.has_identity(&identity) {
            return Err(DomainError::DuplicateIdentity);
        }

        self.identities.push(identity);
        self.updated_at = now;
        Ok(())
    }

    pub fn disable(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.ensure_not_deleted()?;
        self.status = UserStatus::Disabled;
        self.updated_at = now;
        Ok(())
    }

    pub fn soft_delete(&mut self, now: Timestamp) -> Result<(), DomainError> {
        if self.deleted_at.is_some() {
            return Err(DomainError::AlreadyDeleted);
        }

        self.status = UserStatus::Disabled;
        self.updated_at = now;
        self.deleted_at = Some(now);
        Ok(())
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn status(&self) -> UserStatus {
        self.status
    }

    pub fn profile(&self) -> &UserProfile {
        &self.profile
    }

    pub fn identities(&self) -> &[UserIdentity] {
        &self.identities
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn deleted_at(&self) -> Option<Timestamp> {
        self.deleted_at
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn ensure_not_deleted(&self) -> Result<(), DomainError> {
        if self.is_deleted() {
            Err(DomainError::UserDeleted)
        } else {
            Ok(())
        }
    }

    fn has_identity(&self, identity: &UserIdentity) -> bool {
        self.identities.iter().any(|current| {
            current.identity_type() == identity.identity_type()
                && current.identifier_normalized() == identity.identifier_normalized()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdentityType, NormalizedIdentifier};
    use time::macros::datetime;

    #[test]
    fn user_status_rejects_invalid_value() {
        let error = UserStatus::parse("archived").unwrap_err();
        assert_eq!(
            error,
            DomainError::InvalidUserStatus("archived".to_string())
        );
    }

    #[test]
    fn identity_type_rejects_invalid_value() {
        let error = crate::IdentityType::parse("oauth").unwrap_err();
        assert_eq!(error, DomainError::InvalidIdentityType("oauth".to_string()));
    }

    #[test]
    fn normalized_identifier_rejects_empty_value() {
        let error = NormalizedIdentifier::new("   ").unwrap_err();
        assert_eq!(error, DomainError::EmptyIdentifier);
    }

    #[test]
    fn soft_deleted_user_cannot_update_profile() {
        let now = datetime!(2026-03-10 08:00 UTC);
        let later = datetime!(2026-03-10 09:00 UTC);
        let mut user = User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            now,
        );

        user.soft_delete(later).unwrap();

        let error = user
            .update_profile(
                UserProfile::new("Bob", None, None, None).unwrap(),
                datetime!(2026-03-10 10:00 UTC),
            )
            .unwrap_err();

        assert_eq!(error, DomainError::UserDeleted);
    }

    #[test]
    fn user_cannot_bind_duplicate_identity() {
        let now = datetime!(2026-03-10 08:00 UTC);
        let mut user = User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            now,
        );
        let identity = UserIdentity::new(
            IdentityType::Email,
            NormalizedIdentifier::new("Alice@Example.com").unwrap(),
            now,
        );

        user.bind_identity(identity.clone(), now).unwrap();
        let error = user.bind_identity(identity, now).unwrap_err();

        assert_eq!(error, DomainError::DuplicateIdentity);
    }

    #[test]
    fn soft_delete_disables_user() {
        let now = datetime!(2026-03-10 08:00 UTC);
        let mut user = User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            now,
        );

        user.soft_delete(datetime!(2026-03-10 09:00 UTC)).unwrap();

        assert_eq!(user.status(), UserStatus::Disabled);
        assert!(user.deleted_at().is_some());
    }
}
