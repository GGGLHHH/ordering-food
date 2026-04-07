use ordering_food_organization_published::{
    BrandLookupGateway, BrandRef, OrganizationCollaborationError, StoreRef, StoreScopeGateway,
    StoreStatusChanged, StoreSummary,
};
use time::macros::datetime;

#[test]
fn store_summary_contains_stable_scope_fields_for_other_contexts() {
    let summary = StoreSummary {
        store_id: "store-1".to_string(),
        brand_id: "brand-1".to_string(),
        slug: "demo-kitchen".to_string(),
        name: "Demo Kitchen".to_string(),
        currency_code: "CNY".to_string(),
        timezone: "Asia/Shanghai".to_string(),
        status: "active".to_string(),
    };

    let brand_ref = BrandRef {
        brand_id: summary.brand_id.clone(),
    };
    let store_ref = StoreRef {
        store_id: summary.store_id.clone(),
        brand_id: summary.brand_id.clone(),
    };

    assert_eq!(brand_ref.brand_id, "brand-1");
    assert_eq!(store_ref.store_id, "store-1");
}

#[test]
fn organization_scope_contract_keeps_phase_2a_canonical_shape() {
    let brand = BrandRef {
        brand_id: "brand-1".to_string(),
    };
    let store = StoreRef {
        store_id: "store-1".to_string(),
        brand_id: brand.brand_id.clone(),
    };
    let summary = StoreSummary {
        store_id: store.store_id.clone(),
        brand_id: store.brand_id.clone(),
        slug: "demo-kitchen".to_string(),
        name: "Demo Kitchen".to_string(),
        currency_code: "CNY".to_string(),
        timezone: "Asia/Shanghai".to_string(),
        status: "active".to_string(),
    };

    assert_eq!(summary.brand_id, "brand-1");
    assert_eq!(store.store_id, "store-1");
}

#[test]
fn store_status_changed_exposes_stable_event_shape() {
    let event = StoreStatusChanged {
        store_id: "store-1".to_string(),
        brand_id: "brand-1".to_string(),
        previous_status: "inactive".to_string(),
        current_status: "active".to_string(),
        occurred_at: datetime!(2026-04-05 08:02 UTC),
    };

    assert_eq!(event.store_id, "store-1");
    assert_eq!(event.brand_id, "brand-1");
    assert_eq!(event.previous_status, "inactive");
    assert_eq!(event.current_status, "active");
}

#[test]
fn collaboration_errors_keep_semantic_variants() {
    assert_eq!(
        OrganizationCollaborationError::validation("bad brand id").to_string(),
        "validation failed: bad brand id"
    );
    assert_eq!(
        OrganizationCollaborationError::not_found("brand scope was not found").to_string(),
        "resource not found: brand scope was not found"
    );
    assert_eq!(
        OrganizationCollaborationError::conflict("store slug already exists").to_string(),
        "conflict: store slug already exists"
    );
    let unexpected = OrganizationCollaborationError::unexpected("query failed");
    assert_eq!(unexpected.to_string(), "unexpected: query failed");
    assert!(matches!(
        unexpected,
        OrganizationCollaborationError::Unexpected { details: None, .. }
    ));

    let unexpected_with_source =
        OrganizationCollaborationError::unexpected_with_source("query failed", "db timeout");
    assert!(matches!(
        unexpected_with_source,
        OrganizationCollaborationError::Unexpected {
            details: Some(ref source),
            ..
        } if source == "db timeout"
    ));
}

#[test]
fn published_gateway_traits_cover_brand_and_store_scope_queries() {
    fn assert_brand_lookup_gateway<T: BrandLookupGateway + ?Sized>() {}
    fn assert_store_scope_gateway<T: StoreScopeGateway + ?Sized>() {}

    assert_brand_lookup_gateway::<dyn BrandLookupGateway>();
    assert_store_scope_gateway::<dyn StoreScopeGateway>();
}
