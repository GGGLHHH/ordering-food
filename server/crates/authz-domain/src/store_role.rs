#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreRole {
    StoreOwner,
    StoreStaff,
}

impl StoreRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StoreOwner => "store_owner",
            Self::StoreStaff => "store_staff",
        }
    }
}
