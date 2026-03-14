use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("order status `{0}` is invalid")]
    InvalidOrderStatus(String),
    #[error("order must contain at least one item")]
    EmptyOrderItems,
    #[error("item name cannot be empty")]
    EmptyItemName,
    #[error("item quantity must be greater than zero")]
    InvalidItemQuantity,
    #[error("item unit price amount cannot be negative")]
    NegativeUnitPriceAmount,
    #[error("line total amount is inconsistent with unit price and quantity")]
    InvalidLineTotalAmount,
    #[error("subtotal amount is inconsistent with order items")]
    InvalidSubtotalAmount,
    #[error("total amount is inconsistent with order subtotal")]
    InvalidTotalAmount,
    #[error("cannot apply event `{event}` while order is `{status}`")]
    InvalidTransition { event: String, status: String },
}
