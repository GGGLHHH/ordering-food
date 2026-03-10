use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdentityType {
    Email,
    Phone,
}

impl IdentityType {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "email" => Ok(Self::Email),
            "phone" => Ok(Self::Phone),
            other => Err(DomainError::InvalidIdentityType(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Phone => "phone",
        }
    }
}
