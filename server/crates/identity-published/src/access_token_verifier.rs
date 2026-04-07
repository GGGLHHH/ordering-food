use crate::IdentityCollaborationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedSubjectRef {
    subject_id: String,
}

impl AuthenticatedSubjectRef {
    pub fn new(subject_id: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
        }
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }
}

pub trait AccessTokenVerifier: Send + Sync {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedSubjectRef, IdentityCollaborationError>;
}
