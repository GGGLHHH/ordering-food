use crate::{CategoryId, DomainError, MenuStatus, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    id: CategoryId,
    store_id: StoreId,
    slug: String,
    name: String,
    description: Option<String>,
    sort_order: i32,
    status: MenuStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
    deleted_at: Option<Timestamp>,
}

impl Category {
    pub fn create(
        id: CategoryId,
        store_id: StoreId,
        slug: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        sort_order: i32,
        status: MenuStatus,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            store_id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            description: trim_option(description),
            sort_order,
            status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    pub fn rehydrate(
        id: CategoryId,
        store_id: StoreId,
        slug: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        sort_order: i32,
        status: MenuStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            store_id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            description: trim_option(description),
            sort_order,
            status,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    pub fn id(&self) -> &CategoryId {
        &self.id
    }

    pub fn store_id(&self) -> &StoreId {
        &self.store_id
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }

    pub fn status(&self) -> MenuStatus {
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

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
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

fn trim_option(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    })
}
