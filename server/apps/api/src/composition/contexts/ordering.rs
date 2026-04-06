use crate::composition::{
    capabilities::IDENTITY_ACCESS_TOKEN_VERIFIER,
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use crate::routes::ordering::{self, OrderingApiDoc};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_identity_published::AccessTokenVerifier;
use ordering_food_ordering_integration::build_ordering_context_runtime;
use std::sync::Arc;
use utoipa::OpenApi;

pub fn register_ordering() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "ordering",
        depends_on: &["identity"],
    };

    ApiContextRegistration::without_migration(descriptor, ordering_bootstrap_registration)
}

fn ordering_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = platform.clock.clone();
        let token_verifier = platform
            .capabilities
            .resolve::<Arc<dyn AccessTokenVerifier>>(IDENTITY_ACCESS_TOKEN_VERIFIER);
        async move {
            let runtime = build_ordering_context_runtime(pg_pool.clone(), clock.clone());
            let module = runtime.module().clone();
            let token_verifier = token_verifier.ok_or_else(|| {
                std::io::Error::other(
                    "identity capability `identity.access_token_verifier` is not available",
                )
            })?;

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.add_route_mount(
                ordering::ORDER_ROUTE_PREFIX,
                ordering::router(module.clone()).layer(axum::Extension(token_verifier)),
            );
            contribution.add_openapi_document(OrderingApiDoc::openapi());
            contribution.retain_private(module);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
