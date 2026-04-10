//! Published contracts for the Ordering bounded context.

use ordering_food_shared_kernel::Timestamp;
use serde::{Deserialize, Serialize};

pub const COMMERCIAL_ORDER_PLACED_EVENT_TYPE: &str =
    "ordering.commercial_order_placed.v1";
pub const COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE: &str =
    "ordering.commercial_order_status_changed.v1";
pub const COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE: &str =
    "ordering.commercial_order_cancelled_by_customer.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommercialOrderLineSnapshotV1 {
    pub line_number: i32,
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommercialOrderPlacedV1 {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: Timestamp,
    pub items: Vec<CommercialOrderLineSnapshotV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommercialOrderStatusChangedV1 {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub previous_status: String,
    pub current_status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommercialOrderCancelledByCustomerV1 {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: Timestamp,
}
