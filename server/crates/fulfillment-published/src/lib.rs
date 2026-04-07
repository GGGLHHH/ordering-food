//! Published contracts for the Fulfillment bounded context.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FulfillmentOrderRef {
    pub fulfillment_order_id: String,
}
