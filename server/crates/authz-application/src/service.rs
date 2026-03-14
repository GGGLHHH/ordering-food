use crate::ApplicationError;
use async_trait::async_trait;
use ordering_food_authz_domain::{GlobalRole, StoreRole};
use std::sync::Arc;

#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    async fn get_global_roles(&self, user_id: &str) -> Result<Vec<GlobalRole>, ApplicationError>;
    async fn get_store_roles(
        &self,
        user_id: &str,
        store_id: &str,
    ) -> Result<Vec<StoreRole>, ApplicationError>;
}

#[derive(Clone)]
pub struct AuthorizationService {
    repository: Arc<dyn AuthorizationRepository>,
}

impl AuthorizationService {
    pub fn new(repository: Arc<dyn AuthorizationRepository>) -> Self {
        Self { repository }
    }

    pub async fn can_manage_order(
        &self,
        user_id: &str,
        store_id: &str,
    ) -> Result<bool, ApplicationError> {
        let global_roles = self.repository.get_global_roles(user_id).await?;
        if global_roles.contains(&GlobalRole::PlatformAdmin) {
            return Ok(true);
        }

        let store_roles = self.repository.get_store_roles(user_id, store_id).await?;
        Ok(store_roles.contains(&StoreRole::StoreOwner)
            || store_roles.contains(&StoreRole::StoreStaff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::{collections::HashMap, sync::Mutex};

    #[derive(Default)]
    struct FakeAuthorizationRepository {
        global_roles: Mutex<HashMap<String, Vec<GlobalRole>>>,
        store_roles: Mutex<HashMap<(String, String), Vec<StoreRole>>>,
    }

    #[async_trait]
    impl AuthorizationRepository for FakeAuthorizationRepository {
        async fn get_global_roles(
            &self,
            user_id: &str,
        ) -> Result<Vec<GlobalRole>, ApplicationError> {
            Ok(self
                .global_roles
                .lock()
                .unwrap()
                .get(user_id)
                .cloned()
                .unwrap_or_default())
        }

        async fn get_store_roles(
            &self,
            user_id: &str,
            store_id: &str,
        ) -> Result<Vec<StoreRole>, ApplicationError> {
            Ok(self
                .store_roles
                .lock()
                .unwrap()
                .get(&(user_id.to_string(), store_id.to_string()))
                .cloned()
                .unwrap_or_default())
        }
    }

    #[tokio::test]
    async fn platform_admin_can_manage_any_order() {
        let repository = Arc::new(FakeAuthorizationRepository::default());
        repository
            .global_roles
            .lock()
            .unwrap()
            .insert("user-1".to_string(), vec![GlobalRole::PlatformAdmin]);
        let service = AuthorizationService::new(repository);

        assert!(service.can_manage_order("user-1", "store-1").await.unwrap());
    }

    #[tokio::test]
    async fn store_roles_are_scoped_to_store() {
        let repository = Arc::new(FakeAuthorizationRepository::default());
        repository.store_roles.lock().unwrap().insert(
            ("user-1".to_string(), "store-1".to_string()),
            vec![StoreRole::StoreStaff],
        );
        let service = AuthorizationService::new(repository);

        assert!(service.can_manage_order("user-1", "store-1").await.unwrap());
        assert!(!service.can_manage_order("user-1", "store-2").await.unwrap());
    }
}
