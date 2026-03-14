use crate::composition::{
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use ordering_food_authz_application::AuthorizationService;
use ordering_food_authz_infrastructure_sqlx::SqlxAuthorizationRepository;
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use std::sync::Arc;

pub fn register_authz() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "authz",
        depends_on: &[],
    };

    ApiContextRegistration::without_migration(descriptor, authz_bootstrap_registration)
}

fn authz_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let repository = Arc::new(SqlxAuthorizationRepository::new(platform.pg_pool.clone()));
        async move {
            let service = Arc::new(AuthorizationService::new(repository));
            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.retain_private(service);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
