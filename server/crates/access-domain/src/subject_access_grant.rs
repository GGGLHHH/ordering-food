use crate::{AccessRole, AccessScope};
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidSubjectAccessGrant {
    PlatformRoleRequiresPlatformScope,
    StoreRoleRequiresStoreScope,
}

impl fmt::Display for InvalidSubjectAccessGrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlatformRoleRequiresPlatformScope => {
                write!(f, "platform roles require platform scope")
            }
            Self::StoreRoleRequiresStoreScope => {
                write!(f, "store roles require store scope")
            }
        }
    }
}

impl Error for InvalidSubjectAccessGrant {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectAccessGrant {
    subject_id: String,
    scope: AccessScope,
    role: AccessRole,
}

impl SubjectAccessGrant {
    fn new(subject_id: impl Into<String>, scope: AccessScope, role: AccessRole) -> Self {
        Self::try_new(subject_id, scope, role).expect("grant constructors must respect invariants")
    }

    pub fn try_new(
        subject_id: impl Into<String>,
        scope: AccessScope,
        role: AccessRole,
    ) -> Result<Self, InvalidSubjectAccessGrant> {
        if !role.supports_scope(&scope) {
            return Err(match role {
                AccessRole::PlatformAdmin => {
                    InvalidSubjectAccessGrant::PlatformRoleRequiresPlatformScope
                }
                AccessRole::StoreOwner | AccessRole::StoreStaff => {
                    InvalidSubjectAccessGrant::StoreRoleRequiresStoreScope
                }
            });
        }

        let subject_id = subject_id.into();

        Ok(Self {
            subject_id,
            scope,
            role,
        })
    }

    pub fn platform_admin(subject_id: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
            scope: AccessScope::platform(),
            role: AccessRole::PlatformAdmin,
        }
    }

    pub fn store_owner(subject_id: impl Into<String>, store_id: impl Into<String>) -> Self {
        Self::new(
            subject_id,
            AccessScope::store(store_id),
            AccessRole::StoreOwner,
        )
    }

    pub fn store_staff(subject_id: impl Into<String>, store_id: impl Into<String>) -> Self {
        Self::new(
            subject_id,
            AccessScope::store(store_id),
            AccessRole::StoreStaff,
        )
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    pub fn scope(&self) -> &AccessScope {
        &self.scope
    }

    pub fn role(&self) -> AccessRole {
        self.role
    }

    pub fn allows_manage_order(&self, store_id: &str) -> bool {
        match &self.scope {
            AccessScope::Platform => self.role.can_manage_order_in_scope(&self.scope),
            AccessScope::Store {
                store_id: scoped_store_id,
            } => scoped_store_id == store_id && self.role.can_manage_order_in_scope(&self.scope),
        }
    }
}
