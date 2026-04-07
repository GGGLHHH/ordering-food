use crate::AccessScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessRole {
    PlatformAdmin,
    StoreOwner,
    StoreStaff,
}

impl AccessRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlatformAdmin => "platform_admin",
            Self::StoreOwner => "store_owner",
            Self::StoreStaff => "store_staff",
        }
    }

    pub fn can_manage_order(&self) -> bool {
        matches!(
            self,
            Self::PlatformAdmin | Self::StoreOwner | Self::StoreStaff
        )
    }

    pub fn supports_scope(&self, scope: &AccessScope) -> bool {
        matches!(
            (self, scope),
            (Self::PlatformAdmin, AccessScope::Platform)
                | (Self::StoreOwner, AccessScope::Store { .. })
                | (Self::StoreStaff, AccessScope::Store { .. })
        )
    }

    pub fn can_manage_order_in_scope(&self, scope: &AccessScope) -> bool {
        self.supports_scope(scope) && self.can_manage_order()
    }
}
