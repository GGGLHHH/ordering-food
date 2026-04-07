//! Published contracts for the Ordering bounded context.

use ordering_food_shared_kernel::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderPlacedItem {
    pub line_number: i32,
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderPlaced {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub status: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub items: Vec<OrderPlacedItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCommercialStateChanged {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub previous_status: String,
    pub current_status: String,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCancelledByCustomer {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub occurred_at: Timestamp,
}
