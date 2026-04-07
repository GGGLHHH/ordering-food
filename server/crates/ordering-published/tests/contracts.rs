use ordering_food_ordering_published::{
    OrderCancelledByCustomer, OrderCommercialStateChanged, OrderPlaced, OrderPlacedItem,
};
use time::macros::datetime;

#[test]
fn order_placed_exposes_stable_event_shape() {
    let event = OrderPlaced {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        status: "placed".to_string(),
        subtotal_amount: 3200,
        total_amount: 3200,
        created_at: datetime!(2026-04-05 12:00 UTC),
        updated_at: datetime!(2026-04-05 12:00 UTC),
        items: vec![OrderPlacedItem {
            line_number: 1,
            catalog_item_id: "item-1".to_string(),
            name: "Fried Rice".to_string(),
            unit_price_amount: 3200,
            quantity: 1,
            line_total_amount: 3200,
        }],
    };

    assert_eq!(event.order_id, "order-1");
    assert_eq!(event.customer_id, "customer-1");
    assert_eq!(event.items[0].catalog_item_id, "item-1");
}

#[test]
fn order_commercial_state_changed_exposes_status_transition() {
    let event = OrderCommercialStateChanged {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        previous_status: "placed".to_string(),
        current_status: "cancelled_by_customer".to_string(),
        occurred_at: datetime!(2026-04-05 12:05 UTC),
    };

    assert_eq!(event.previous_status, "placed");
    assert_eq!(event.current_status, "cancelled_by_customer");
}

#[test]
fn order_cancelled_by_customer_exposes_stable_event_shape() {
    let event = OrderCancelledByCustomer {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        occurred_at: datetime!(2026-04-05 12:05 UTC),
    };

    assert_eq!(event.order_id, "order-1");
    assert_eq!(event.customer_id, "customer-1");
    assert_eq!(event.store_id, "store-1");
}
