use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("workflow status `{0}` is invalid")]
    InvalidWorkflowStatus(String),
    #[error("cannot apply event `{event}` while workflow order is `{status}`")]
    InvalidTransition { event: String, status: String },
}
