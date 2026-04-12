use super::{catalog, organization};
use crate::composition::{
    capabilities::{ORGANIZATION_BRAND_LOOKUP_GATEWAY, ORGANIZATION_STORE_SCOPE_GATEWAY},
    platform::ApiPlatform,
};
use async_trait::async_trait;
use ordering_food_organization_published::{
    BrandLookupGateway, BrandRef, OrganizationCollaborationError, StoreScopeGateway, StoreSummary,
};
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};

fn test_platform() -> ApiPlatform {
    ApiPlatform::new(
        crate::config::Settings::from_overrides(std::iter::empty::<(String, String)>()).unwrap(),
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(50))
            .connect_lazy("postgres://ordering_food:ordering_food@127.0.0.1:9/ordering_food")
            .unwrap(),
        redis::Client::open("redis://127.0.0.1:6379").unwrap(),
    )
}

struct StaticBrandLookupGateway;

#[async_trait]
impl BrandLookupGateway for StaticBrandLookupGateway {
    async fn get_by_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandRef>, OrganizationCollaborationError> {
        Ok(Some(BrandRef {
            brand_id: brand_id.to_string(),
        }))
    }
}

struct StaticStoreScopeGateway;

#[async_trait]
impl StoreScopeGateway for StaticStoreScopeGateway {
    async fn get_active(&self) -> Result<Option<StoreSummary>, OrganizationCollaborationError> {
        Ok(Some(StoreSummary {
            store_id: "10000000-0000-4000-8000-000000000001".to_string(),
            brand_id: "00000000-0000-4000-8000-000000000001".to_string(),
            slug: "ordering-food-demo".to_string(),
            name: "Ordering Food Demo".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        }))
    }

    async fn get_by_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreSummary>, OrganizationCollaborationError> {
        Ok(Some(StoreSummary {
            store_id: store_id.to_string(),
            brand_id: "00000000-0000-4000-8000-000000000001".to_string(),
            slug: "ordering-food-demo".to_string(),
            name: "Ordering Food Demo".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        }))
    }
}

#[tokio::test]
async fn organization_bootstrap_registration_skips_seed_in_http_path_by_default() {
    let platform = test_platform();

    let contribution = organization::register_organization()
        .bootstrap_registration()
        .run(&platform)
        .await
        .expect("organization bootstrap should succeed without running seed");

    let parts = contribution.into_parts();

    assert_eq!(parts.context_id, "organization");
    assert_eq!(parts.readiness_checks.len(), 1);
    assert_eq!(parts.private_runtime_objects.len(), 1);
}

#[tokio::test]
async fn catalog_bootstrap_registration_skips_seed_in_http_path_by_default() {
    let platform = test_platform();
    platform.capabilities.publish(
        ORGANIZATION_BRAND_LOOKUP_GATEWAY,
        Arc::new(StaticBrandLookupGateway) as Arc<dyn BrandLookupGateway>,
    );
    platform.capabilities.publish(
        ORGANIZATION_STORE_SCOPE_GATEWAY,
        Arc::new(StaticStoreScopeGateway) as Arc<dyn StoreScopeGateway>,
    );

    let contribution = catalog::register_catalog()
        .bootstrap_registration()
        .run(&platform)
        .await
        .expect("catalog bootstrap should succeed without running seed");

    let parts = contribution.into_parts();

    assert_eq!(parts.context_id, "catalog");
    assert_eq!(parts.route_mounts.len(), 1);
    assert_eq!(parts.openapi_documents.len(), 1);
    assert_eq!(parts.readiness_checks.len(), 1);
    assert_eq!(parts.private_runtime_objects.len(), 1);
}
