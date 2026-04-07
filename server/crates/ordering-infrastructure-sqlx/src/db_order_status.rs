use ordering_food_ordering_domain::OrderStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "ordering.order_status", rename_all = "snake_case")]
pub enum DbOrderStatus {
    Placed,
    CancelledByCustomer,
}

impl From<OrderStatus> for DbOrderStatus {
    fn from(value: OrderStatus) -> Self {
        match value {
            OrderStatus::Placed => Self::Placed,
            OrderStatus::CancelledByCustomer => Self::CancelledByCustomer,
        }
    }
}

impl From<DbOrderStatus> for OrderStatus {
    fn from(value: DbOrderStatus) -> Self {
        match value {
            DbOrderStatus::Placed => OrderStatus::Placed,
            DbOrderStatus::CancelledByCustomer => OrderStatus::CancelledByCustomer,
        }
    }
}
