use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("menu status `{0}` is invalid")]
    InvalidMenuStatus(String),
    #[error("slug cannot be empty")]
    EmptySlug,
    #[error("name cannot be empty")]
    EmptyName,
    #[error("currency code must be a 3-letter uppercase ISO code")]
    InvalidCurrencyCode,
    #[error("timezone cannot be empty")]
    EmptyTimezone,
    #[error("price amount cannot be negative")]
    NegativePriceAmount,
}
