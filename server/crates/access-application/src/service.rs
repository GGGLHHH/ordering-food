use crate::{
    AccessGrantRepository, AccessSubjectStatus, ApplicationError, StoreScopeFactsPort,
    SubjectFactsPort,
};
use ordering_food_access_domain::AccessScope;
use std::sync::Arc;

#[derive(Clone)]
pub struct AccessService {
    grant_repository: Arc<dyn AccessGrantRepository>,
    subject_facts_port: Arc<dyn SubjectFactsPort>,
    store_scope_facts_port: Arc<dyn StoreScopeFactsPort>,
}

impl AccessService {
    pub fn new(
        grant_repository: Arc<dyn AccessGrantRepository>,
        subject_facts_port: Arc<dyn SubjectFactsPort>,
        store_scope_facts_port: Arc<dyn StoreScopeFactsPort>,
    ) -> Self {
        Self {
            grant_repository,
            subject_facts_port,
            store_scope_facts_port,
        }
    }

    pub async fn can_manage_order(
        &self,
        subject_id: &str,
        store_id: &str,
    ) -> Result<bool, ApplicationError> {
        let subject = match self.subject_facts_port.get_subject(subject_id).await? {
            Some(subject) if subject.status() == AccessSubjectStatus::Active => subject,
            _ => return Ok(false),
        };

        let store = match self.store_scope_facts_port.get_store(store_id).await? {
            Some(store) if store.store_id() == store_id => store,
            _ => return Ok(false),
        };

        let platform_roles = self
            .grant_repository
            .get_platform_roles(subject.subject_id())
            .await?;
        let platform_scope = AccessScope::platform();
        if platform_roles
            .iter()
            .any(|role| role.can_manage_order_in_scope(&platform_scope))
        {
            return Ok(true);
        }

        let store_roles = self
            .grant_repository
            .get_store_roles(subject.subject_id(), store.store_id())
            .await?;
        let store_scope = AccessScope::store(store.store_id().to_string());

        Ok(store_roles
            .iter()
            .any(|role| role.can_manage_order_in_scope(&store_scope)))
    }
}
