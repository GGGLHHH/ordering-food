use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// Domain errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum UserDomainError {
    #[error("invalid phone number: {0}")]
    InvalidPhone(String),
}

// ---------------------------------------------------------------------------
// Value objects
// ---------------------------------------------------------------------------

/// Validated phone number (10-15 digits).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phone(String);

impl Phone {
    pub fn new(raw: &str) -> Result<Self, UserDomainError> {
        let cleaned: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() < 10 || cleaned.len() > 15 {
            return Err(UserDomainError::InvalidPhone(format!(
                "phone must be 10-15 digits, got {}",
                cleaned.len()
            )));
        }
        Ok(Self(cleaned))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// User role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Customer,
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Customer => "customer",
            Self::Admin => "admin",
        }
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "customer" => Some(Self::Customer),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }
}

/// User account status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
}

impl UserStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "inactive" => Some(Self::Inactive),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Entity (aggregate root)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub phone: String,
    pub nickname: String,
    pub avatar_url: String,
    pub role: Role,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Creation / update parameters
// ---------------------------------------------------------------------------

pub struct NewUser {
    pub phone: Phone,
}

#[derive(Debug, Default)]
pub struct UpdateUser {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<Role>,
    pub status: Option<UserStatus>,
}
