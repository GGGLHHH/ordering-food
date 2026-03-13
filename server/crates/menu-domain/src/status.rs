use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuStatus {
    Active,
    Inactive,
}

impl MenuStatus {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            other => Err(DomainError::InvalidMenuStatus(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}
