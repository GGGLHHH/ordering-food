use crate::{BrandRef, StoreSummary};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OrganizationCollaborationError {
    #[error("validation failed: {message}")]
    Validation { message: String },
    #[error("resource not found: {message}")]
    NotFound { message: String },
    #[error("conflict: {message}")]
    Conflict { message: String },
    #[error("unexpected: {message}")]
    Unexpected {
        message: String,
        details: Option<String>,
    },
}

impl OrganizationCollaborationError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
            details: None,
        }
    }

    pub fn unexpected_with_source(message: impl Into<String>, source: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
            details: Some(source.into()),
        }
    }
}

#[async_trait]
pub trait BrandLookupGateway: Send + Sync {
    async fn get_by_id(
        &self,
        brand_id: &str,
    ) -> Result<Option<BrandRef>, OrganizationCollaborationError>;
}

#[async_trait]
pub trait StoreScopeGateway: Send + Sync {
    async fn get_active(&self) -> Result<Option<StoreSummary>, OrganizationCollaborationError>;

    async fn get_by_id(
        &self,
        store_id: &str,
    ) -> Result<Option<StoreSummary>, OrganizationCollaborationError>;
}
