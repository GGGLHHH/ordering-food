use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreStatusChanged {
    pub store_id: String,
    pub brand_id: String,
    pub previous_status: String,
    pub current_status: String,
    pub occurred_at: Timestamp,
}
