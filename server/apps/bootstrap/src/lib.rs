use anyhow::{Context, Result};
use ordering_food_app_support::config::Settings;
use ordering_food_catalog_integration::{
    CatalogBootstrap, build_catalog_context_runtime, seed_default_catalog,
};
use ordering_food_organization_integration::{
    build_organization_context_runtime, resolve_seeded_store_scope, seed_default_organization,
};
use ordering_food_platform_kernel::Clock;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

pub async fn run_default_data_bootstrap(
    settings: &Settings,
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
) -> Result<()> {
    let organization_runtime = build_organization_context_runtime(pg_pool.clone(), clock.clone());
    let store_scope_gateway = organization_runtime.store_scope_gateway().clone();
    let brand_lookup_gateway = organization_runtime.brand_lookup_gateway().clone();

    let organization_outcome = seed_default_organization(&organization_runtime)
        .await
        .context("failed to seed default organization")?;
    let seeded_store = resolve_seeded_store_scope(&organization_runtime, &organization_outcome)
        .await
        .context("failed to resolve seeded organization store")?;

    let catalog_runtime =
        build_catalog_context_runtime(pg_pool, brand_lookup_gateway, store_scope_gateway, clock);
    let catalog_bootstrap = CatalogBootstrap {
        brand_slug: settings.catalog.bootstrap.brand_slug.clone(),
        brand_name: settings.catalog.bootstrap.brand_name.clone(),
    };
    let catalog_outcome = seed_default_catalog(&catalog_runtime, seeded_store, catalog_bootstrap)
        .await
        .context("failed to seed default catalog")?;

    info!(
        ?organization_outcome,
        ?catalog_outcome,
        "default bootstrap completed"
    );

    Ok(())
}
