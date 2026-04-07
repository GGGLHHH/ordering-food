mod access;
mod catalog;
mod database;
mod fulfillment;
mod identity;
mod ordering;
mod organization;

use crate::composition::context_registration::ApiContextRegistration;

pub fn registrations() -> Vec<ApiContextRegistration> {
    vec![
        database::register_database(),
        identity::register_identity(),
        access::register_access(),
        organization::register_organization(),
        catalog::register_catalog(),
        ordering::register_ordering(),
        fulfillment::register_fulfillment(),
    ]
}
