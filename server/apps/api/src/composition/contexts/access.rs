use crate::composition::{
    capabilities::{
        ACCESS_ORDER_MANAGEMENT_GATEWAY, IDENTITY_SUBJECT_LOOKUP_GATEWAY,
        ORGANIZATION_STORE_SCOPE_GATEWAY,
    },
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use ordering_food_access_integration::build_access_context_runtime;
use ordering_food_access_published::OrderManagementAccessGateway;
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_identity_published::SubjectLookupGateway;
use ordering_food_organization_published::StoreScopeGateway;
use std::sync::Arc;

pub fn register_access() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "access",
        depends_on: &["identity", "organization"],
    };

    ApiContextRegistration::without_migration(descriptor, access_bootstrap_registration)
}

fn access_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let subject_gateway = platform
            .capabilities
            .resolve::<Arc<dyn SubjectLookupGateway>>(IDENTITY_SUBJECT_LOOKUP_GATEWAY);
        let store_scope_gateway = platform
            .capabilities
            .resolve::<Arc<dyn StoreScopeGateway>>(ORGANIZATION_STORE_SCOPE_GATEWAY);
        let pg_pool = platform.pg_pool.clone();
        let capabilities = platform.capabilities.clone();

        async move {
            let subject_gateway = subject_gateway.ok_or_else(|| {
                std::io::Error::other(
                    "identity capability `identity.subject_lookup_gateway` is not available",
                )
            })?;
            let store_scope_gateway = store_scope_gateway.ok_or_else(|| {
                std::io::Error::other(
                    "organization capability `organization.store_scope_gateway` is not available",
                )
            })?;
            let access_runtime =
                build_access_context_runtime(pg_pool, subject_gateway, store_scope_gateway);
            let order_management_gateway: Arc<dyn OrderManagementAccessGateway> =
                access_runtime.order_management_gateway().clone();
            capabilities.publish(ACCESS_ORDER_MANAGEMENT_GATEWAY, order_management_gateway);

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.retain_private(access_runtime);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
