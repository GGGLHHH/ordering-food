use ordering_food_authz_domain::{GlobalRole, StoreRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "authz.global_role", rename_all = "snake_case")]
pub enum DbGlobalRole {
    PlatformAdmin,
}

impl From<DbGlobalRole> for GlobalRole {
    fn from(value: DbGlobalRole) -> Self {
        match value {
            DbGlobalRole::PlatformAdmin => Self::PlatformAdmin,
        }
    }
}

impl From<GlobalRole> for DbGlobalRole {
    fn from(value: GlobalRole) -> Self {
        match value {
            GlobalRole::PlatformAdmin => Self::PlatformAdmin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "authz.store_role", rename_all = "snake_case")]
pub enum DbStoreRole {
    StoreOwner,
    StoreStaff,
}

impl From<DbStoreRole> for StoreRole {
    fn from(value: DbStoreRole) -> Self {
        match value {
            DbStoreRole::StoreOwner => Self::StoreOwner,
            DbStoreRole::StoreStaff => Self::StoreStaff,
        }
    }
}

impl From<StoreRole> for DbStoreRole {
    fn from(value: StoreRole) -> Self {
        match value {
            StoreRole::StoreOwner => Self::StoreOwner,
            StoreRole::StoreStaff => Self::StoreStaff,
        }
    }
}
