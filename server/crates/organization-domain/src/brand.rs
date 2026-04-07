use crate::{BrandId, DomainError, OrganizationStatus};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brand {
    id: BrandId,
    slug: String,
    name: String,
    status: OrganizationStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
    deleted_at: Option<Timestamp>,
}

impl Brand {
    pub fn create(
        id: BrandId,
        slug: impl Into<String>,
        name: impl Into<String>,
        status: OrganizationStatus,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: BrandId,
        slug: impl Into<String>,
        name: impl Into<String>,
        status: OrganizationStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            status,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    pub fn id(&self) -> &BrandId {
        &self.id
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> OrganizationStatus {
        self.status
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn deleted_at(&self) -> Option<Timestamp> {
        self.deleted_at
    }
}

fn normalize_slug(value: impl Into<String>) -> Result<String, DomainError> {
    let value = value.into().trim().to_ascii_lowercase();
    if value.is_empty() {
        return Err(DomainError::EmptySlug);
    }
    Ok(value)
}

fn normalize_name(value: impl Into<String>) -> Result<String, DomainError> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(DomainError::EmptyName);
    }
    Ok(value)
}
