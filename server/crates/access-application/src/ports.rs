use crate::{AccessStoreScopeFacts, AccessSubjectFacts, ApplicationError};
use async_trait::async_trait;
use ordering_food_access_domain::AccessRole;

#[async_trait]
pub trait AccessGrantRepository: Send + Sync {
    async fn get_platform_roles(
        &self,
        subject_id: &str,
    ) -> Result<Vec<AccessRole>, ApplicationError>;

    async fn get_store_roles(
        &self,
        subject_id: &str,
        store_id: &str,
    ) -> Result<Vec<AccessRole>, ApplicationError>;
}

#[async_trait]
pub trait SubjectFactsPort: Send + Sync {
    async fn get_subject(
        &self,
        subject_id: &str,
    ) -> Result<Option<AccessSubjectFacts>, ApplicationError>;
}

#[async_trait]
pub trait StoreScopeFactsPort: Send + Sync {
    async fn get_store(
        &self,
        store_id: &str,
    ) -> Result<Option<AccessStoreScopeFacts>, ApplicationError>;
}
