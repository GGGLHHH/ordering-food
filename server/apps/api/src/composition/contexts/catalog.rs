use crate::{
    composition::{
        capabilities::{ORGANIZATION_BRAND_LOOKUP_GATEWAY, ORGANIZATION_STORE_SCOPE_GATEWAY},
        context_registration::ApiContextRegistration,
        contribution::{ApiContextContribution, ApiNamedReadinessCheck},
        platform::ApiPlatform,
    },
    routes::catalog::{self, CatalogApiDoc},
};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_catalog_integration::{
    CatalogBootstrap, build_catalog_context_runtime, seed_default_catalog,
};
use ordering_food_organization_published::{BrandLookupGateway, StoreScopeGateway};
use std::sync::Arc;
use utoipa::OpenApi;

pub fn register_catalog() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "catalog",
        depends_on: &["organization"],
    };

    ApiContextRegistration::without_migration(descriptor, catalog_bootstrap_registration)
}

fn catalog_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = platform.clock.clone();
        let brand_lookup_gateway = platform
            .capabilities
            .resolve::<Arc<dyn BrandLookupGateway>>(ORGANIZATION_BRAND_LOOKUP_GATEWAY);
        let store_scope_gateway = platform
            .capabilities
            .resolve::<Arc<dyn StoreScopeGateway>>(ORGANIZATION_STORE_SCOPE_GATEWAY);
        let bootstrap = CatalogBootstrap {
            brand_slug: platform.settings.catalog.bootstrap.brand_slug.clone(),
            brand_name: platform.settings.catalog.bootstrap.brand_name.clone(),
        };

        async move {
            let brand_lookup_gateway = brand_lookup_gateway.ok_or_else(|| {
                std::io::Error::other(
                    "organization capability `organization.brand_lookup_gateway` is not available",
                )
            })?;
            let store_scope_gateway = store_scope_gateway.ok_or_else(|| {
                std::io::Error::other(
                    "organization capability `organization.store_scope_gateway` is not available",
                )
            })?;
            let catalog_runtime = build_catalog_context_runtime(
                pg_pool,
                brand_lookup_gateway,
                store_scope_gateway.clone(),
                clock,
            );
            seed_default_catalog(&catalog_runtime, store_scope_gateway, bootstrap)
                .await
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let catalog_module = catalog_runtime.module().clone();

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.add_route_mount(
                catalog::CATALOG_ROUTE_PREFIX,
                catalog::router(catalog_module.clone()),
            );
            contribution.add_openapi_document(CatalogApiDoc::openapi());
            contribution.retain_private(catalog_runtime);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}
