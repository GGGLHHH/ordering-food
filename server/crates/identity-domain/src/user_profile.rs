use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    display_name: String,
    given_name: Option<String>,
    family_name: Option<String>,
    avatar_url: Option<String>,
}

impl UserProfile {
    pub fn new(
        display_name: impl Into<String>,
        given_name: Option<String>,
        family_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<Self, DomainError> {
        let display_name = display_name.into().trim().to_string();
        if display_name.is_empty() {
            return Err(DomainError::EmptyDisplayName);
        }

        Ok(Self {
            display_name,
            given_name: given_name.and_then(trim_option),
            family_name: family_name.and_then(trim_option),
            avatar_url: avatar_url.and_then(trim_option),
        })
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn given_name(&self) -> Option<&str> {
        self.given_name.as_deref()
    }

    pub fn family_name(&self) -> Option<&str> {
        self.family_name.as_deref()
    }

    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_deref()
    }
}

fn trim_option(value: String) -> Option<String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
