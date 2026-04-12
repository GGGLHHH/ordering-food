use crate::composition::{
    capabilities::{ORGANIZATION_BRAND_LOOKUP_GATEWAY, ORGANIZATION_STORE_SCOPE_GATEWAY},
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_organization_integration::build_organization_context_runtime;

pub fn register_organization() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "organization",
        depends_on: &["database"],
    };

    ApiContextRegistration::without_migration(descriptor, organization_bootstrap_registration)
}

fn organization_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = platform.clock.clone();
        let capabilities = platform.capabilities.clone();
        async move {
            let runtime = build_organization_context_runtime(pg_pool, clock);
            capabilities.publish(
                ORGANIZATION_STORE_SCOPE_GATEWAY,
                runtime.store_scope_gateway().clone(),
            );
            capabilities.publish(
                ORGANIZATION_BRAND_LOOKUP_GATEWAY,
                runtime.brand_lookup_gateway().clone(),
            );

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.retain_private(runtime);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
