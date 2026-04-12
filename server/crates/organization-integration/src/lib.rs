use async_trait::async_trait;
use ordering_food_organization_application::{
    ApplicationError as OrganizationApplicationError, BrandQueryService,
    BrandReadModel as OrganizationBrandReadModel, EnsureDefaultOrganizationInput,
    EnsureDefaultOrganizationOutcome, IdGenerator as OrganizationIdGenerator, OrganizationModule,
    StoreQueryService, StoreReadModel as OrganizationStoreReadModel,
};
use ordering_food_organization_domain::{BrandId, StoreId};
use ordering_food_organization_infrastructure_sqlx::build_organization_module;
use ordering_food_organization_published::{
    BrandLookupGateway, BrandRef, OrganizationCollaborationError, StoreScopeGateway, StoreSummary,
};
use ordering_food_platform_kernel::Clock;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const DEFAULT_BRAND_ID: &str = "00000000-0000-4000-8000-000000000001";

#[derive(Clone)]
pub struct OrganizationContextRuntime {
    module: Arc<OrganizationModule>,
    store_scope_gateway: Arc<dyn StoreScopeGateway>,
    brand_lookup_gateway: Arc<dyn BrandLookupGateway>,
}

impl OrganizationContextRuntime {
    pub fn module(&self) -> &Arc<OrganizationModule> {
        &self.module
    }

    pub fn store_scope_gateway(&self) -> &Arc<dyn StoreScopeGateway> {
        &self.store_scope_gateway
    }

    pub fn brand_lookup_gateway(&self) -> &Arc<dyn BrandLookupGateway> {
        &self.brand_lookup_gateway
    }
}

pub fn build_organization_context_runtime(
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
) -> OrganizationContextRuntime {
    let module = build_organization_module(pg_pool, clock, Arc::new(UuidV4OrganizationIdGenerator));
    let store_scope_gateway: Arc<dyn StoreScopeGateway> =
        Arc::new(SqlxStoreScopeGateway::new(module.store_queries().clone()));
    let brand_lookup_gateway: Arc<dyn BrandLookupGateway> =
        Arc::new(SqlxBrandLookupGateway::new(module.brand_queries().clone()));
    OrganizationContextRuntime {
        module,
        store_scope_gateway,
        brand_lookup_gateway,
    }
}

pub async fn seed_default_organization(
    runtime: &OrganizationContextRuntime,
) -> Result<EnsureDefaultOrganizationOutcome, OrganizationApplicationError> {
    let outcome = runtime
        .module
        .ensure_default_organization()
        .execute(default_organization_input())
        .await?;

    match &outcome {
        EnsureDefaultOrganizationOutcome::Skipped { store_id, slug } => {
            info!(
                store_id = %store_id,
                slug = %slug,
                "organization seed skipped because the default store already exists"
            );
        }
        EnsureDefaultOrganizationOutcome::CreatedStore { store_id, brand_id } => {
            info!(
                store_id = %store_id,
                brand_id = %brand_id,
                "organization seed created default store"
            );
        }
        EnsureDefaultOrganizationOutcome::CreatedBrandAndStore { store_id, brand_id } => {
            info!(
                store_id = %store_id,
                brand_id = %brand_id,
                "organization seed created default brand and store"
            );
        }
        EnsureDefaultOrganizationOutcome::RecoveredStore { store_id, brand_id } => {
            info!(
                store_id = %store_id,
                brand_id = %brand_id,
                "organization seed recovered default store from inactive or deleted state"
            );
        }
    }

    Ok(outcome)
}

pub async fn resolve_seeded_store_scope(
    runtime: &OrganizationContextRuntime,
    outcome: &EnsureDefaultOrganizationOutcome,
) -> Result<StoreSummary, OrganizationApplicationError> {
    let store_id = match outcome {
        EnsureDefaultOrganizationOutcome::Skipped { store_id, .. }
        | EnsureDefaultOrganizationOutcome::CreatedStore { store_id, .. }
        | EnsureDefaultOrganizationOutcome::CreatedBrandAndStore { store_id, .. }
        | EnsureDefaultOrganizationOutcome::RecoveredStore { store_id, .. } => store_id,
    };

    runtime
        .module
        .store_queries()
        .get_by_id(store_id)
        .await?
        .map(map_store_summary)
        .ok_or_else(|| {
            OrganizationApplicationError::not_found("seeded organization store was not found")
        })
}

struct SqlxStoreScopeGateway {
    store_queries: Arc<StoreQueryService>,
}

impl SqlxStoreScopeGateway {
    fn new(store_queries: Arc<StoreQueryService>) -> Self {
        Self { store_queries }
    }
}

struct SqlxBrandLookupGateway {
    brand_queries: Arc<BrandQueryService>,
}

impl SqlxBrandLookupGateway {
    fn new(brand_queries: Arc<BrandQueryService>) -> Self {
        Self { brand_queries }
    }
}

#[async_trait]
impl StoreScopeGateway for SqlxStoreScopeGateway {
    async fn get_active(&self) -> Result<Option<StoreSummary>, OrganizationCollaborationError> {
        let store = self
            .store_queries
            .get_active()
            .await
            .map_err(map_organization_application_error)?;
        Ok(store.map(map_store_summary))
    }

    async fn get_by_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreSummary>, OrganizationCollaborationError> {
        let store = self
            .store_queries
            .get_by_id(store_id)
            .await
            .map_err(map_organization_application_error)?;
        Ok(store.map(map_store_summary))
    }
}

#[async_trait]
impl BrandLookupGateway for SqlxBrandLookupGateway {
    async fn get_by_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandRef>, OrganizationCollaborationError> {
        let brand = self
            .brand_queries
            .get_by_id(brand_id)
            .await
            .map_err(map_organization_application_error)?;
        Ok(brand.map(map_brand_ref))
    }
}

fn default_organization_input() -> EnsureDefaultOrganizationInput {
    EnsureDefaultOrganizationInput {
        brand_id: DEFAULT_BRAND_ID.to_string(),
        brand_slug: "ordering-food".to_string(),
        brand_name: "Ordering Food".to_string(),
        brand_status: "active".to_string(),
        store_slug: "ordering-food-demo".to_string(),
        store_name: "Ordering Food Demo Kitchen".to_string(),
        store_currency_code: "CNY".to_string(),
        store_timezone: "Asia/Shanghai".to_string(),
        store_status: "active".to_string(),
    }
}

fn map_brand_ref(brand: OrganizationBrandReadModel) -> BrandRef {
    BrandRef {
        brand_id: brand.brand_id,
    }
}

fn map_store_summary(store: OrganizationStoreReadModel) -> StoreSummary {
    StoreSummary {
        store_id: store.store_id,
        brand_id: store.brand_id,
        slug: store.slug,
        name: store.name,
        currency_code: store.currency_code,
        timezone: store.timezone,
        status: store.status,
    }
}

fn map_organization_application_error(
    error: OrganizationApplicationError,
) -> OrganizationCollaborationError {
    match error {
        OrganizationApplicationError::Validation { message } => {
            OrganizationCollaborationError::validation(message)
        }
        OrganizationApplicationError::NotFound { message } => {
            OrganizationCollaborationError::not_found(message)
        }
        OrganizationApplicationError::Conflict { message } => {
            OrganizationCollaborationError::conflict(message)
        }
        OrganizationApplicationError::Unexpected { message, source } => match source {
            Some(source) => OrganizationCollaborationError::unexpected_with_source(
                message,
                format_error_chain(source.as_ref()),
            ),
            None => OrganizationCollaborationError::unexpected(message),
        },
    }
}

fn format_error_chain(error: &dyn std::error::Error) -> String {
    let mut chain = vec![error.to_string()];
    let mut source = error.source();
    while let Some(next) = source {
        chain.push(next.to_string());
        source = next.source();
    }
    chain.join(": ")
}

struct UuidV4OrganizationIdGenerator;

impl OrganizationIdGenerator for UuidV4OrganizationIdGenerator {
    fn next_brand_id(&self) -> BrandId {
        BrandId::new(Uuid::new_v4().to_string())
    }

    fn next_store_id(&self) -> StoreId {
        StoreId::new(Uuid::new_v4().to_string())
    }
}
