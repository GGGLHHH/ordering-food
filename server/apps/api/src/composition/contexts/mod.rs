mod database;
mod identity;
mod menu;

use crate::composition::context_registration::ApiContextRegistration;

pub fn registrations() -> Vec<ApiContextRegistration> {
    vec![
        database::register_database(),
        identity::register_identity(),
        menu::register_menu(),
    ]
}
