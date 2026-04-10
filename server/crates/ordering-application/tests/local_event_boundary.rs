#[test]
fn ordering_application_does_not_depend_on_published_event_contracts() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let lib = std::fs::read_to_string("src/lib.rs").unwrap();
    let module = std::fs::read_to_string("src/module.rs").unwrap();
    let ordering_events = std::fs::read_to_string("src/ordering_events.rs").unwrap();
    let ports = std::fs::read_to_string("src/ports.rs").unwrap();
    let place_order = std::fs::read_to_string("src/use_cases/place_order_from_cart.rs").unwrap();
    let cancel_order =
        std::fs::read_to_string("src/use_cases/cancel_order_by_customer.rs").unwrap();

    assert!(!manifest.contains("ordering-food-ordering-published"));
    assert!(!lib.contains(" OrderPlaced,"));
    assert!(!lib.contains(" OrderCommercialStateChanged,"));
    assert!(!lib.contains(" OrderCancelledByCustomer,"));
    assert!(!lib.contains("OrderingPublishedEventRecorder"));
    assert!(!module.contains("ordering_food_ordering_published"));
    assert!(!module.contains("OrderingPublishedEventRecorder"));
    assert!(!ordering_events.contains("ordering_food_ordering_published"));
    assert!(!ordering_events.contains(" OrderPlaced,"));
    assert!(!ordering_events.contains(" OrderCommercialStateChanged,"));
    assert!(!ordering_events.contains(" OrderCancelledByCustomer,"));
    assert!(!ports.contains("ordering_food_ordering_published"));
    assert!(!ports.contains("OrderingPublishedEventRecorder"));
    assert!(!place_order.contains("ordering_food_ordering_published"));
    assert!(!cancel_order.contains("ordering_food_ordering_published"));
}
