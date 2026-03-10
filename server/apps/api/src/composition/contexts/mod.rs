mod identity;

use crate::composition::context_registration::ApiContextRegistration;

pub fn registrations() -> Vec<ApiContextRegistration> {
    vec![identity::register_identity()]
}
