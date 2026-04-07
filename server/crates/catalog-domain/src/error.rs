use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("slug cannot be empty")]
    EmptySlug,
    #[error("name cannot be empty")]
    EmptyName,
    #[error("price amount cannot be negative")]
    NegativePriceAmount,
    #[error("invalid sellable status: `{0}`")]
    InvalidSellableStatus(String),
    #[error("invalid display rule: `{0}`")]
    InvalidDisplayRule(String),
}
