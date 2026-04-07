use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("organization status `{0}` is invalid")]
    InvalidOrganizationStatus(String),
    #[error("slug cannot be empty")]
    EmptySlug,
    #[error("name cannot be empty")]
    EmptyName,
    #[error("currency code must be a 3-letter uppercase ISO code")]
    InvalidCurrencyCode,
    #[error("timezone cannot be empty")]
    EmptyTimezone,
}
