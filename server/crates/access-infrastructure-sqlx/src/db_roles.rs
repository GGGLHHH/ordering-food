use ordering_food_access_domain::AccessRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "access.global_role", rename_all = "snake_case")]
pub enum DbGlobalRole {
    PlatformAdmin,
}

impl From<DbGlobalRole> for AccessRole {
    fn from(value: DbGlobalRole) -> Self {
        match value {
            DbGlobalRole::PlatformAdmin => Self::PlatformAdmin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "access.store_role", rename_all = "snake_case")]
pub enum DbStoreRole {
    StoreOwner,
    StoreStaff,
}

impl From<DbStoreRole> for AccessRole {
    fn from(value: DbStoreRole) -> Self {
        match value {
            DbStoreRole::StoreOwner => Self::StoreOwner,
            DbStoreRole::StoreStaff => Self::StoreStaff,
        }
    }
}
