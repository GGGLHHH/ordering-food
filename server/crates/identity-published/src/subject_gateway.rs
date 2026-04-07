use crate::SubjectRef;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct IdentityCollaborationError {
    message: String,
}

impl IdentityCollaborationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[async_trait]
pub trait SubjectLookupGateway: Send + Sync {
    async fn get_by_id(
        &self,
        subject_id: &str,
    ) -> Result<Option<SubjectRef>, IdentityCollaborationError>;
}
