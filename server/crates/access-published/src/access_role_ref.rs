#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessRoleRef {
    PlatformAdmin,
    StoreOwner,
    StoreStaff,
}

impl AccessRoleRef {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlatformAdmin => "platform_admin",
            Self::StoreOwner => "store_owner",
            Self::StoreStaff => "store_staff",
        }
    }
}
