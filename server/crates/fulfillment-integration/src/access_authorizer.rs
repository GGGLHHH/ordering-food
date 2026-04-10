use async_trait::async_trait;
use ordering_food_access_published::OrderManagementAccessGateway;
use ordering_food_fulfillment_application::{
    ApplicationError, WorkflowAction, WorkflowActionAuthorizer,
};
use std::sync::Arc;

pub struct AccessWorkflowActionAuthorizer {
    gateway: Arc<dyn OrderManagementAccessGateway>,
}

impl AccessWorkflowActionAuthorizer {
    pub fn new(gateway: Arc<dyn OrderManagementAccessGateway>) -> Self {
        Self { gateway }
    }
}

#[async_trait]
impl WorkflowActionAuthorizer for AccessWorkflowActionAuthorizer {
    async fn ensure_actor_can_manage_order(
        &self,
        actor_user_id: &str,
        store_id: &str,
        _action: WorkflowAction,
    ) -> Result<(), ApplicationError> {
        let allowed = self
            .gateway
            .can_manage_order(actor_user_id, store_id)
            .await
            .map_err(|error| {
                ApplicationError::unexpected_with_source("failed to query access", error)
            })?;

        if allowed {
            Ok(())
        } else {
            Err(ApplicationError::not_found("order was not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordering_food_access_published::{AccessCollaborationError, OrderManagementAccessGateway};

    enum GatewayBehavior {
        Allow,
        Deny,
        Error,
    }

    struct FakeOrderManagementAccessGateway {
        behavior: GatewayBehavior,
    }

    #[async_trait]
    impl OrderManagementAccessGateway for FakeOrderManagementAccessGateway {
        async fn can_manage_order(
            &self,
            _subject_id: &str,
            _store_id: &str,
        ) -> Result<bool, AccessCollaborationError> {
            match self.behavior {
                GatewayBehavior::Allow => Ok(true),
                GatewayBehavior::Deny => Ok(false),
                GatewayBehavior::Error => Err(AccessCollaborationError::new("gateway failed")),
            }
        }
    }

    #[tokio::test]
    async fn allow_returns_ok() {
        let gateway: Arc<dyn OrderManagementAccessGateway> =
            Arc::new(FakeOrderManagementAccessGateway {
                behavior: GatewayBehavior::Allow,
            });
        let authorizer = AccessWorkflowActionAuthorizer::new(gateway);

        authorizer
            .ensure_actor_can_manage_order("user-1", "store-1", WorkflowAction::Accept)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn deny_returns_not_found_with_public_message() {
        let gateway: Arc<dyn OrderManagementAccessGateway> =
            Arc::new(FakeOrderManagementAccessGateway {
                behavior: GatewayBehavior::Deny,
            });
        let authorizer = AccessWorkflowActionAuthorizer::new(gateway);

        let error = authorizer
            .ensure_actor_can_manage_order("user-1", "store-1", WorkflowAction::Accept)
            .await
            .unwrap_err();

        match error {
            ApplicationError::NotFound { message } => {
                assert_eq!(message, "order was not found");
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn gateway_error_maps_to_unexpected() {
        let gateway: Arc<dyn OrderManagementAccessGateway> =
            Arc::new(FakeOrderManagementAccessGateway {
                behavior: GatewayBehavior::Error,
            });
        let authorizer = AccessWorkflowActionAuthorizer::new(gateway);

        let error = authorizer
            .ensure_actor_can_manage_order("user-1", "store-1", WorkflowAction::Accept)
            .await
            .unwrap_err();

        match error {
            ApplicationError::Unexpected { message, source } => {
                assert_eq!(message, "failed to query access");
                assert!(source.is_some());
            }
            other => panic!("expected Unexpected, got {other:?}"),
        }
    }
}
