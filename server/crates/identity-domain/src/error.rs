use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("user status `{0}` is invalid")]
    InvalidUserStatus(String),
    #[error("identity type `{0}` is invalid")]
    InvalidIdentityType(String),
    #[error("identifier cannot be empty")]
    EmptyIdentifier,
    #[error("display name cannot be empty")]
    EmptyDisplayName,
    #[error("identity is already bound to the user")]
    DuplicateIdentity,
    #[error("soft deleted user cannot be mutated")]
    UserDeleted,
    #[error("user is already soft deleted")]
    AlreadyDeleted,
    #[error("soft deleted user must be disabled")]
    DeletedUserMustBeDisabled,
}
