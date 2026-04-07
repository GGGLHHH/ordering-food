use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus, Store, StoreId};
use time::macros::datetime;

#[test]
fn store_belongs_to_brand_and_preserves_scope_fields() {
    let brand = Brand::create(
        BrandId::new("brand-1"),
        "ordering-food",
        "Ordering Food",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:00 UTC),
    )
    .unwrap();

    let store = Store::create(
        StoreId::new("store-1"),
        brand.id().clone(),
        "demo-kitchen",
        "Demo Kitchen",
        "CNY",
        "Asia/Shanghai",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:01 UTC),
    )
    .unwrap();

    assert_eq!(store.brand_id(), brand.id());
    assert_eq!(store.currency_code(), "CNY");
    assert_eq!(store.timezone(), "Asia/Shanghai");
}

#[test]
fn organization_status_rejects_invalid_value() {
    let error = OrganizationStatus::parse("archived").unwrap_err();

    assert_eq!(
        error,
        ordering_food_organization_domain::DomainError::InvalidOrganizationStatus(
            "archived".to_string()
        )
    );
}

#[test]
fn brand_rejects_empty_slug() {
    let error = Brand::create(
        BrandId::new("brand-1"),
        "   ",
        "Ordering Food",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:00 UTC),
    )
    .unwrap_err();

    assert_eq!(
        error,
        ordering_food_organization_domain::DomainError::EmptySlug
    );
}

#[test]
fn store_rejects_invalid_currency_code() {
    let error = Store::create(
        StoreId::new("store-1"),
        BrandId::new("brand-1"),
        "demo-kitchen",
        "Demo Kitchen",
        "rmbb",
        "Asia/Shanghai",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:01 UTC),
    )
    .unwrap_err();

    assert_eq!(
        error,
        ordering_food_organization_domain::DomainError::InvalidCurrencyCode
    );
}

#[test]
fn store_rejects_empty_timezone() {
    let error = Store::create(
        StoreId::new("store-1"),
        BrandId::new("brand-1"),
        "demo-kitchen",
        "Demo Kitchen",
        "CNY",
        "   ",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:01 UTC),
    )
    .unwrap_err();

    assert_eq!(
        error,
        ordering_food_organization_domain::DomainError::EmptyTimezone
    );
}
