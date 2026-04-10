use ordering_food_ordering_published::{
    CommercialOrderCancelledByCustomerV1, CommercialOrderLineSnapshotV1,
    CommercialOrderPlacedV1, CommercialOrderStatusChangedV1,
    COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE,
    COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
    COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE,
};
use serde_json::json;
use time::macros::datetime;

#[test]
fn commercial_order_placed_v1_exposes_stable_event_shape() {
    let event = CommercialOrderPlacedV1 {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        subtotal_amount: 3200,
        total_amount: 3200,
        occurred_at: datetime!(2026-04-05 12:00 UTC),
        items: vec![CommercialOrderLineSnapshotV1 {
            line_number: 1,
            catalog_item_id: "item-1".to_string(),
            name: "Fried Rice".to_string(),
            unit_price_amount: 3200,
            quantity: 1,
            line_total_amount: 3200,
        }],
    };

    let value = serde_json::to_value(&event).unwrap();

    assert_eq!(
        value,
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

    let object = value.as_object().unwrap();
    assert!(!object.contains_key("status"));
    assert!(!object.contains_key("created_at"));
    assert!(!object.contains_key("updated_at"));
}

#[test]
fn commercial_order_status_changed_v1_exposes_status_transition() {
    let event = CommercialOrderStatusChangedV1 {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        previous_status: "placed".to_string(),
        current_status: "cancelled_by_customer".to_string(),
        occurred_at: datetime!(2026-04-05 12:05 UTC),
    };

    assert_eq!(
        serde_json::to_value(&event).unwrap(),
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
fn commercial_order_cancelled_by_customer_v1_exposes_stable_event_shape() {
    let event = CommercialOrderCancelledByCustomerV1 {
        order_id: "order-1".to_string(),
        customer_id: "customer-1".to_string(),
        store_id: "store-1".to_string(),
        occurred_at: datetime!(2026-04-05 12:05 UTC),
    };

    assert_eq!(
        serde_json::to_value(&event).unwrap(),
        json!({
            "order_id": "order-1",
            "customer_id": "customer-1",
            "store_id": "store-1",
            "occurred_at": "2026-04-05T12:05:00Z"
        })
    );
}

#[test]
fn commercial_event_type_constants_are_locked_to_contract_strings() {
    assert_eq!(
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        "ordering.commercial_order_placed.v1"
    );
    assert_eq!(
        COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE,
        "ordering.commercial_order_status_changed.v1"
    );
    assert_eq!(
        COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE,
        "ordering.commercial_order_cancelled_by_customer.v1"
    );
}
