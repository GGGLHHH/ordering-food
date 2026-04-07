mod dto;
mod module;
mod ports;
pub mod use_cases;

use ordering_food_organization_domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("validation failed: {message}")]
    Validation { message: String },
    #[error("resource not found: {message}")]
    NotFound { message: String },
    #[error("conflict: {message}")]
    Conflict { message: String },
    #[error("unexpected: {message}")]
    Unexpected {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
}

impl ApplicationError {
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
            source: None,
        }
    }

    pub fn unexpected_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Unexpected {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<DomainError> for ApplicationError {
    fn from(value: DomainError) -> Self {
        Self::validation(value.to_string())
    }
}

pub use dto::{BrandRef, StoreSummary};
pub use module::OrganizationModule;
pub use ports::{
    BrandQueryService, BrandReadRepository, Clock, IdGenerator, OrganizationUnitOfWork,
    OrganizationUnitOfWorkFactory, StoreQueryService, StoreReadRepository,
};
pub use use_cases::{
    CreateBrand, CreateBrandInput, CreateStore, CreateStoreInput, EnsureDefaultOrganization,
    EnsureDefaultOrganizationInput, EnsureDefaultOrganizationOutcome,
};
