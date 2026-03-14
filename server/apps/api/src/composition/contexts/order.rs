use crate::composition::{
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use crate::routes::orders::{self, OrderApiDoc};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_identity_application::TokenService;
use ordering_food_identity_infrastructure_auth::JwtTokenService;
use ordering_food_order_application::{Clock as OrderClock, IdGenerator as OrderIdGenerator};
use ordering_food_order_domain::OrderId;
use ordering_food_order_infrastructure_sqlx::build_order_module;
use ordering_food_shared_kernel::Timestamp;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

pub fn register_order() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "order",
        depends_on: &[],
    };

    ApiContextRegistration::without_migration(descriptor, order_bootstrap_registration)
}

fn order_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = Arc::new(OrderClockAdapter {
            inner: platform.clock.clone(),
        });
        let id_generator = Arc::new(UuidV4OrderIdGenerator);
        let auth_settings = platform.settings.auth.clone();
        async move {
            let module = build_order_module(pg_pool, clock, id_generator);
            let token_service: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(
                auth_settings.jwt_secret.clone(),
                auth_settings.access_token_ttl_seconds,
                auth_settings.refresh_token_ttl_seconds,
            ));

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.add_route_mount(
                orders::ORDER_ROUTE_PREFIX,
                orders::router(module.clone()).layer(axum::Extension(token_service)),
            );
            contribution.add_openapi_document(OrderApiDoc::openapi());
            contribution.retain_private(module);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}

struct OrderClockAdapter {
    inner: Arc<dyn ordering_food_identity_application::Clock>,
}

impl OrderClock for OrderClockAdapter {
    fn now(&self) -> Timestamp {
        self.inner.now()
    }
}

struct UuidV4OrderIdGenerator;

impl OrderIdGenerator for UuidV4OrderIdGenerator {
    fn next_order_id(&self) -> OrderId {
        OrderId::new(Uuid::new_v4().to_string())
    }
}
