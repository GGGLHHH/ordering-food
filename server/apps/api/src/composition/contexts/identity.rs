use crate::composition::{
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use crate::routes::auth::{self, AuthApiDoc};
use crate::routes::identity::{self, IdentityApiDoc};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_identity_application::TokenService;
use ordering_food_identity_infrastructure_auth::{
    Argon2PasswordHasher, JwtTokenService, RedisRefreshTokenStore,
};
use ordering_food_identity_infrastructure_sqlx::build_identity_module;
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
        let id_generator = platform.id_generator.clone();
        let auth_settings = platform.settings.auth.clone();
        let redis_client = platform.redis_client.clone();
        async move {
            let password_hasher = Arc::new(Argon2PasswordHasher);
            let token_service: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(
                auth_settings.jwt_secret.clone(),
                auth_settings.access_token_ttl_seconds,
                auth_settings.refresh_token_ttl_seconds,
            ));
            let refresh_token_store = Arc::new(RedisRefreshTokenStore::new(redis_client));

            let module = build_identity_module(
                pg_pool,
                clock,
                id_generator,
                password_hasher,
                token_service.clone(),
                refresh_token_store,
            );

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
                auth::router(module.clone(), auth_settings).layer(axum::Extension(token_service)),
            );
            contribution.add_openapi_document(IdentityApiDoc::openapi());
            contribution.add_openapi_document(AuthApiDoc::openapi());
            contribution.retain_private(module);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
