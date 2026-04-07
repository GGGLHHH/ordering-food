use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct AccessCollaborationError {
    message: String,
}

impl AccessCollaborationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[async_trait]
pub trait OrderManagementAccessGateway: Send + Sync {
    async fn can_manage_order(
        &self,
        subject_id: &str,
        store_id: &str,
    ) -> Result<bool, AccessCollaborationError>;
}
