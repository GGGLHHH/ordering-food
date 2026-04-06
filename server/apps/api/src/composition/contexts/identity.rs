use crate::composition::{
    capabilities::{IDENTITY_ACCESS_TOKEN_VERIFIER, IDENTITY_SUBJECT_LOOKUP_GATEWAY},
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use crate::routes::auth::{self, AuthApiDoc};
use crate::routes::identity::{self, IdentityApiDoc};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_identity_integration::{IdentityContextConfig, build_identity_context_runtime};
use ordering_food_identity_published::{AccessTokenVerifier, SubjectLookupGateway};
use std::sync::Arc;
use utoipa::OpenApi;

pub fn register_identity() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "identity",
        depends_on: &[],
    };

    ApiContextRegistration::without_migration(descriptor, identity_bootstrap_registration)
}

fn identity_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = platform.clock.clone();
        let auth_settings = platform.settings.auth.clone();
        let redis_client = platform.redis_client.clone();
        let capabilities = platform.capabilities.clone();
        async move {
            let runtime = build_identity_context_runtime(
                pg_pool.clone(),
                clock,
                redis_client,
                IdentityContextConfig::new(
                    auth_settings.jwt_secret.clone(),
                    auth_settings.access_token_ttl_seconds,
                    auth_settings.refresh_token_ttl_seconds,
                ),
            );
            let module = runtime.module().clone();
            let token_verifier: Arc<dyn AccessTokenVerifier> =
                runtime.access_token_verifier().clone();
            let subject_lookup_gateway: Arc<dyn SubjectLookupGateway> =
                runtime.subject_lookup_gateway().clone();

            capabilities.publish(IDENTITY_ACCESS_TOKEN_VERIFIER, token_verifier.clone());
            capabilities.publish(IDENTITY_SUBJECT_LOOKUP_GATEWAY, subject_lookup_gateway);

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.add_route_mount(
                identity::IDENTITY_ROUTE_PREFIX,
                identity::router(module.clone()),
            );
            contribution.add_route_mount(
                auth::AUTH_ROUTE_PREFIX,
                auth::router(module.clone(), auth_settings).layer(axum::Extension(token_verifier)),
            );
            contribution.add_openapi_document(IdentityApiDoc::openapi());
            contribution.add_openapi_document(AuthApiDoc::openapi());
            contribution.retain_private(runtime);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
