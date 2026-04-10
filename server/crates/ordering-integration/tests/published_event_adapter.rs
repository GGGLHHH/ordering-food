use ordering_food_ordering_application::{
    LocalCommercialOrderCancelledByCustomer, LocalCommercialOrderLineSnapshot,
    LocalCommercialOrderPlaced, LocalCommercialOrderStatusChanged, OrderingEvent,
};
use ordering_food_ordering_integration::published_event_adapter::map_ordering_event_to_published;
use serde_json::json;
use time::macros::datetime;

#[test]
fn maps_commercial_order_placed_to_published_outbox_event() {
    let occurred_at = datetime!(2026-04-05 12:00 UTC);
    let event = OrderingEvent::CommercialOrderPlaced(LocalCommercialOrderPlaced {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        subtotal_amount: 3200,
        total_amount: 3200,
        occurred_at,
        items: vec![LocalCommercialOrderLineSnapshot {
            line_number: 1,
            catalog_item_id: "item-1".to_string(),
            name: "Fried Rice".to_string(),
            unit_price_amount: 3200,
            quantity: 1,
            line_total_amount: 3200,
        }],
    });

    let published = map_ordering_event_to_published(&event).unwrap();

    assert_eq!(published.event_type, "ordering.commercial_order_placed.v1");
    assert_eq!(published.aggregate_id, "order-1");
    assert_eq!(published.occurred_at, occurred_at);
    assert_eq!(
        published.payload,
        json!({
            "order_id": "order-1",
            "customer_id": "customer-1",
            "store_id": "store-1",
            "subtotal_amount": 3200,
            "total_amount": 3200,
            "occurred_at": "2026-04-05T12:00:00Z",
            "items": [{
                "line_number": 1,
                "catalog_item_id": "item-1",
                "name": "Fried Rice",
                "unit_price_amount": 3200,
                "quantity": 1,
                "line_total_amount": 3200
            }]
        })
    );
}

#[test]
fn maps_commercial_order_status_changed_to_published_outbox_event() {
    let occurred_at = datetime!(2026-04-05 12:05 UTC);
    let event = OrderingEvent::CommercialOrderStatusChanged(LocalCommercialOrderStatusChanged {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        previous_status: "placed".to_string(),
        current_status: "cancelled_by_customer".to_string(),
        occurred_at,
    });

    let published = map_ordering_event_to_published(&event).unwrap();

    assert_eq!(
        published.event_type,
        "ordering.commercial_order_status_changed.v1"
    );
    assert_eq!(published.aggregate_id, "order-1");
    assert_eq!(published.occurred_at, occurred_at);
    assert_eq!(
        published.payload,
        json!({
            "order_id": "order-1",
            "customer_id": "customer-1",
            "store_id": "store-1",
            "previous_status": "placed",
            "current_status": "cancelled_by_customer",
            "occurred_at": "2026-04-05T12:05:00Z"
        })
    );
}

#[test]
fn maps_commercial_order_cancelled_by_customer_to_published_outbox_event() {
    let occurred_at = datetime!(2026-04-05 12:05 UTC);
    let event = OrderingEvent::CommercialOrderCancelledByCustomer(
        LocalCommercialOrderCancelledByCustomer {
            order_id: "order-1".to_string(),
            customer_id: "customer-1".to_string(),
            store_id: "store-1".to_string(),
            occurred_at,
        },
    );

    let published = map_ordering_event_to_published(&event).unwrap();

    assert_eq!(
        published.event_type,
        "ordering.commercial_order_cancelled_by_customer.v1"
    );
    assert_eq!(published.aggregate_id, "order-1");
    assert_eq!(published.occurred_at, occurred_at);
    assert_eq!(
        published.payload,
        json!({
            "order_id": "order-1",
            "customer_id": "customer-1",
            "store_id": "store-1",
            "occurred_at": "2026-04-05T12:05:00Z"
        })
    );
}
