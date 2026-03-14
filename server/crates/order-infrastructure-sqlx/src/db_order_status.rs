use ordering_food_order_domain::OrderStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "ordering.order_status", rename_all = "snake_case")]
pub enum DbOrderStatus {
    PendingAcceptance,
    Accepted,
    Preparing,
    ReadyForPickup,
    Completed,
    CancelledByCustomer,
    RejectedByStore,
}

impl From<OrderStatus> for DbOrderStatus {
    fn from(value: OrderStatus) -> Self {
        match value {
            OrderStatus::PendingAcceptance => Self::PendingAcceptance,
            OrderStatus::Accepted => Self::Accepted,
            OrderStatus::Preparing => Self::Preparing,
            OrderStatus::ReadyForPickup => Self::ReadyForPickup,
            OrderStatus::Completed => Self::Completed,
            OrderStatus::CancelledByCustomer => Self::CancelledByCustomer,
            OrderStatus::RejectedByStore => Self::RejectedByStore,
        }
    }
}

impl From<DbOrderStatus> for OrderStatus {
    fn from(value: DbOrderStatus) -> Self {
        match value {
            DbOrderStatus::PendingAcceptance => Self::PendingAcceptance,
            DbOrderStatus::Accepted => Self::Accepted,
            DbOrderStatus::Preparing => Self::Preparing,
            DbOrderStatus::ReadyForPickup => Self::ReadyForPickup,
            DbOrderStatus::Completed => Self::Completed,
            DbOrderStatus::CancelledByCustomer => Self::CancelledByCustomer,
            DbOrderStatus::RejectedByStore => Self::RejectedByStore,
        }
    }
}
