use crate::ApplicationError;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowAction {
    Accept,
    StartPreparing,
    MarkReady,
    Complete,
    Reject,
}

#[async_trait]
pub trait WorkflowActionAuthorizer: Send + Sync {
    async fn ensure_actor_can_manage_order(
        &self,
        actor_user_id: &str,
        store_id: &str,
        action: WorkflowAction,
    ) -> Result<(), ApplicationError>;
}
