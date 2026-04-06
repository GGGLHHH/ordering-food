use async_trait::async_trait;
use ordering_food_access_application::{
    AccessService, AccessStoreScopeFacts, AccessSubjectFacts, AccessSubjectStatus,
    ApplicationError as AccessApplicationError, StoreScopeFactsPort, SubjectFactsPort,
};
use ordering_food_access_infrastructure_sqlx::SqlxAccessGrantRepository;
use ordering_food_access_published::{AccessCollaborationError, OrderManagementAccessGateway};
use ordering_food_identity_published::{SubjectLookupGateway, SubjectRef, SubjectStatus};
use ordering_food_organization_published::{StoreScopeGateway, StoreSummary};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AccessContextRuntime {
    service: Arc<AccessService>,
    order_management_gateway: Arc<dyn OrderManagementAccessGateway>,
}

impl AccessContextRuntime {
    pub fn service(&self) -> &Arc<AccessService> {
        &self.service
    }

    pub fn order_management_gateway(&self) -> &Arc<dyn OrderManagementAccessGateway> {
        &self.order_management_gateway
    }
}

pub fn build_access_service(
    pg_pool: PgPool,
    subject_gateway: Arc<dyn SubjectLookupGateway>,
    store_gateway: Arc<dyn StoreScopeGateway>,
) -> Arc<AccessService> {
    let grant_repository = Arc::new(SqlxAccessGrantRepository::new(pg_pool));
    let subject_facts = Arc::new(IdentitySubjectFactsAcl::new(subject_gateway));
    let store_scope_facts = Arc::new(OrganizationStoreScopeFactsAcl::new(store_gateway));

    Arc::new(AccessService::new(
        grant_repository,
        subject_facts,
        store_scope_facts,
    ))
}

pub fn build_access_context_runtime(
    pg_pool: PgPool,
    subject_gateway: Arc<dyn SubjectLookupGateway>,
    store_gateway: Arc<dyn StoreScopeGateway>,
) -> AccessContextRuntime {
    let service = build_access_service(pg_pool, subject_gateway, store_gateway);
    let order_management_gateway: Arc<dyn OrderManagementAccessGateway> =
        Arc::new(AccessDecisionGatewayAcl::new(service.clone()));

    AccessContextRuntime {
        service,
        order_management_gateway,
    }
}

struct AccessDecisionGatewayAcl {
    access_service: Arc<AccessService>,
}

impl AccessDecisionGatewayAcl {
    fn new(access_service: Arc<AccessService>) -> Self {
        Self { access_service }
    }
}

#[async_trait]
impl OrderManagementAccessGateway for AccessDecisionGatewayAcl {
    async fn can_manage_order(
        &self,
        subject_id: &str,
        store_id: &str,
    ) -> Result<bool, AccessCollaborationError> {
        self.access_service
            .can_manage_order(subject_id, store_id)
            .await
            .map_err(|error| AccessCollaborationError::new(error.to_string()))
    }
}

struct IdentitySubjectFactsAcl {
    subject_gateway: Arc<dyn SubjectLookupGateway>,
}

impl IdentitySubjectFactsAcl {
    fn new(subject_gateway: Arc<dyn SubjectLookupGateway>) -> Self {
        Self { subject_gateway }
    }
}

#[async_trait]
impl SubjectFactsPort for IdentitySubjectFactsAcl {
    async fn get_subject(
        &self,
        subject_id: &str,
    ) -> Result<Option<AccessSubjectFacts>, AccessApplicationError> {
        let subject = self
            .subject_gateway
            .get_by_id(subject_id)
            .await
            .map_err(|error| {
                AccessApplicationError::unexpected_with_source(
                    "failed to query identity subject",
                    error,
                )
            })?;

        Ok(subject.map(translate_subject_facts))
    }
}

struct OrganizationStoreScopeFactsAcl {
    store_gateway: Arc<dyn StoreScopeGateway>,
}

impl OrganizationStoreScopeFactsAcl {
    fn new(store_gateway: Arc<dyn StoreScopeGateway>) -> Self {
        Self { store_gateway }
    }
}

#[async_trait]
impl StoreScopeFactsPort for OrganizationStoreScopeFactsAcl {
    async fn get_store(
        &self,
        store_id: &str,
    ) -> Result<Option<AccessStoreScopeFacts>, AccessApplicationError> {
        let store = self
            .store_gateway
            .get_by_id(store_id)
            .await
            .map_err(|error| {
                AccessApplicationError::unexpected_with_source(
                    "failed to query organization store",
                    error,
                )
            })?;

        Ok(store.map(translate_store_scope_facts))
    }
}

fn translate_subject_facts(subject: SubjectRef) -> AccessSubjectFacts {
    let status = match subject.status() {
        SubjectStatus::Active => AccessSubjectStatus::Active,
        SubjectStatus::Disabled => AccessSubjectStatus::Disabled,
    };

    AccessSubjectFacts::new(subject.subject_id(), status)
}

fn translate_store_scope_facts(store: StoreSummary) -> AccessStoreScopeFacts {
    AccessStoreScopeFacts::new(store.store_id, store.brand_id)
}
