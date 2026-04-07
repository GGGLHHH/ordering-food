use ordering_food_access_domain::{AccessRole, AccessScope, SubjectAccessGrant};

#[test]
fn platform_admin_grant_uses_platform_scope() {
    let grant = SubjectAccessGrant::platform_admin("subject-1");

    assert_eq!(grant.subject_id(), "subject-1");
    assert_eq!(grant.role(), AccessRole::PlatformAdmin);
    assert!(grant.scope().is_platform());
    assert!(grant.allows_manage_order("store-1"));
}

#[test]
fn store_scoped_grants_only_match_their_store() {
    let owner_grant = SubjectAccessGrant::store_owner("subject-1", "store-1");
    let staff_grant = SubjectAccessGrant::store_staff("subject-1", "store-1");

    assert_eq!(owner_grant.role(), AccessRole::StoreOwner);
    assert!(owner_grant.scope().matches_store("store-1"));
    assert!(!owner_grant.scope().matches_store("store-2"));
    assert!(owner_grant.allows_manage_order("store-1"));
    assert!(!owner_grant.allows_manage_order("store-2"));

    assert_eq!(staff_grant.role(), AccessRole::StoreStaff);
    assert!(staff_grant.allows_manage_order("store-1"));
    assert!(!staff_grant.allows_manage_order("store-2"));
}

#[test]
fn invalid_role_scope_combinations_are_rejected() {
    let platform_owner =
        SubjectAccessGrant::try_new("subject-1", AccessScope::platform(), AccessRole::StoreOwner);
    let store_admin = SubjectAccessGrant::try_new(
        "subject-1",
        AccessScope::store("store-1"),
        AccessRole::PlatformAdmin,
    );

    assert!(platform_owner.is_err());
    assert!(store_admin.is_err());
}
