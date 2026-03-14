#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalRole {
    PlatformAdmin,
}

impl GlobalRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlatformAdmin => "platform_admin",
        }
    }
}
