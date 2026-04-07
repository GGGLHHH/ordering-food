use ordering_food_access_published::{AccessRoleRef, StoreMembershipRef};

#[test]
fn access_role_ref_uses_stable_role_language() {
    assert_eq!(AccessRoleRef::PlatformAdmin.as_str(), "platform_admin");
    assert_eq!(AccessRoleRef::StoreOwner.as_str(), "store_owner");
    assert_eq!(AccessRoleRef::StoreStaff.as_str(), "store_staff");
}

#[test]
fn store_membership_ref_tracks_subject_store_and_role() {
    let membership = StoreMembershipRef::new("subject-1", "store-1", AccessRoleRef::StoreStaff);

    assert_eq!(membership.subject_id(), "subject-1");
    assert_eq!(membership.store_id(), "store-1");
    assert_eq!(membership.role(), AccessRoleRef::StoreStaff);
}
