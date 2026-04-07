use async_trait::async_trait;
use ordering_food_access_application::{
    AccessGrantRepository, AccessService, AccessStoreScopeFacts, AccessSubjectFacts,
    AccessSubjectStatus, ApplicationError, StoreScopeFactsPort, SubjectFactsPort,
};
use ordering_food_access_domain::SubjectAccessGrant;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[tokio::test]
async fn disabled_subject_cannot_manage_order() {
    let service = build_service_with(
        vec![SubjectAccessGrant::store_staff("subject-1", "store-1")],
        AccessSubjectFacts::new("subject-1", AccessSubjectStatus::Disabled),
        Some(AccessStoreScopeFacts::new("store-1", "brand-1")),
    );

    assert!(
        !service
            .can_manage_order("subject-1", "store-1")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn platform_admin_can_manage_any_store_order() {
    let service = build_service_with(
        vec![SubjectAccessGrant::platform_admin("subject-1")],
        AccessSubjectFacts::new("subject-1", AccessSubjectStatus::Active),
        Some(AccessStoreScopeFacts::new("store-1", "brand-1")),
    );

    assert!(
        service
            .can_manage_order("subject-1", "store-1")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn store_staff_is_scoped_to_matching_store() {
    let service = build_service_with(
        vec![SubjectAccessGrant::store_staff("subject-1", "store-1")],
        AccessSubjectFacts::new("subject-1", AccessSubjectStatus::Active),
        Some(AccessStoreScopeFacts::new("store-1", "brand-1")),
    );

    assert!(
        service
            .can_manage_order("subject-1", "store-1")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn store_staff_is_denied_for_non_matching_store() {
    let service = build_service_with(
        vec![SubjectAccessGrant::store_staff("subject-1", "store-1")],
        AccessSubjectFacts::new("subject-1", AccessSubjectStatus::Active),
        Some(AccessStoreScopeFacts::new("store-2", "brand-1")),
    );

    assert!(
        !service
            .can_manage_order("subject-1", "store-2")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn missing_subject_or_store_facts_deny_access() {
    let missing_subject = build_service(
        Arc::new(FakeGrantRepository::default()),
        Arc::new(FakeSubjectFactsPort::default()),
        Arc::new(FakeStoreScopeFactsPort {
            stores: Mutex::new(HashMap::from([(
                "store-1".to_string(),
                AccessStoreScopeFacts::new("store-1", "brand-1"),
            )])),
        }),
    );
    let missing_store = build_service_with(
        vec![SubjectAccessGrant::platform_admin("subject-1")],
        AccessSubjectFacts::new("subject-1", AccessSubjectStatus::Active),
        None,
    );

    assert!(
        !missing_subject
            .can_manage_order("subject-1", "store-1")
            .await
            .unwrap()
    );
    assert!(
        !missing_store
            .can_manage_order("subject-1", "store-1")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn subject_fact_errors_are_propagated() {
    let service = build_service(
        Arc::new(FakeGrantRepository::default()),
        Arc::new(FakeSubjectFactsPort {
            subjects: Mutex::new(HashMap::new()),
            error_message: Some("subject provider unavailable".to_string()),
        }),
        Arc::new(FakeStoreScopeFactsPort::default()),
    );

    let error = service
        .can_manage_order("subject-1", "store-1")
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        ApplicationError::Unexpected { ref message, .. }
        if message == "subject provider unavailable"
    ));
}

fn build_service_with(
    grants: Vec<SubjectAccessGrant>,
    subject: AccessSubjectFacts,
    store: Option<AccessStoreScopeFacts>,
) -> AccessService {
    let stores = store
        .into_iter()
        .map(|store| (store.store_id().to_string(), store))
        .collect::<HashMap<_, _>>();

    build_service(
        Arc::new(FakeGrantRepository::new(grants)),
        Arc::new(FakeSubjectFactsPort {
            subjects: Mutex::new(HashMap::from([(subject.subject_id().to_string(), subject)])),
            error_message: None,
        }),
        Arc::new(FakeStoreScopeFactsPort {
            stores: Mutex::new(stores),
        }),
    )
}

fn build_service(
    grants: Arc<FakeGrantRepository>,
    subjects: Arc<FakeSubjectFactsPort>,
    stores: Arc<FakeStoreScopeFactsPort>,
) -> AccessService {
    AccessService::new(grants, subjects, stores)
}

#[derive(Default)]
struct FakeGrantRepository {
    grants: Mutex<Vec<SubjectAccessGrant>>,
}

impl FakeGrantRepository {
    fn new(grants: Vec<SubjectAccessGrant>) -> Self {
        Self {
            grants: Mutex::new(grants),
        }
    }
}

#[async_trait]
impl AccessGrantRepository for FakeGrantRepository {
    async fn get_platform_roles(
        &self,
        subject_id: &str,
    ) -> Result<Vec<ordering_food_access_domain::AccessRole>, ApplicationError> {
        Ok(self
            .grants
            .lock()
            .unwrap()
            .iter()
            .filter(|grant| grant.subject_id() == subject_id && grant.scope().is_platform())
            .map(|grant| grant.role())
            .collect())
    }

    async fn get_store_roles(
        &self,
        subject_id: &str,
        store_id: &str,
    ) -> Result<Vec<ordering_food_access_domain::AccessRole>, ApplicationError> {
        Ok(self
            .grants
            .lock()
            .unwrap()
            .iter()
            .filter(|grant| {
                grant.subject_id() == subject_id && grant.scope().matches_store(store_id)
            })
            .map(|grant| grant.role())
            .collect())
    }
}

#[derive(Default)]
struct FakeSubjectFactsPort {
    subjects: Mutex<HashMap<String, AccessSubjectFacts>>,
    error_message: Option<String>,
}

#[async_trait]
impl SubjectFactsPort for FakeSubjectFactsPort {
    async fn get_subject(
        &self,
        subject_id: &str,
    ) -> Result<Option<AccessSubjectFacts>, ApplicationError> {
        if let Some(message) = &self.error_message {
            return Err(ApplicationError::unexpected(message.clone()));
        }

        Ok(self.subjects.lock().unwrap().get(subject_id).cloned())
    }
}

#[derive(Default)]
struct FakeStoreScopeFactsPort {
    stores: Mutex<HashMap<String, AccessStoreScopeFacts>>,
}

#[async_trait]
impl StoreScopeFactsPort for FakeStoreScopeFactsPort {
    async fn get_store(
        &self,
        store_id: &str,
    ) -> Result<Option<AccessStoreScopeFacts>, ApplicationError> {
        Ok(self.stores.lock().unwrap().get(store_id).cloned())
    }
}
