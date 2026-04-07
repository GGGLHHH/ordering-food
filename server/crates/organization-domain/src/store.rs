use crate::{BrandId, DomainError, OrganizationStatus, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    id: StoreId,
    brand_id: BrandId,
    slug: String,
    name: String,
    currency_code: String,
    timezone: String,
    status: OrganizationStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
    deleted_at: Option<Timestamp>,
}

impl Store {
    pub fn normalize_slug(value: impl Into<String>) -> Result<String, DomainError> {
        normalize_slug(value)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: StoreId,
        brand_id: BrandId,
        slug: impl Into<String>,
        name: impl Into<String>,
        currency_code: impl Into<String>,
        timezone: impl Into<String>,
        status: OrganizationStatus,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            brand_id,
            slug: Self::normalize_slug(slug)?,
            name: normalize_name(name)?,
            currency_code: normalize_currency_code(currency_code)?,
            timezone: normalize_timezone(timezone)?,
            status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: StoreId,
        brand_id: BrandId,
        slug: impl Into<String>,
        name: impl Into<String>,
        currency_code: impl Into<String>,
        timezone: impl Into<String>,
        status: OrganizationStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            brand_id,
            slug: Self::normalize_slug(slug)?,
            name: normalize_name(name)?,
            currency_code: normalize_currency_code(currency_code)?,
            timezone: normalize_timezone(timezone)?,
            status,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    pub fn id(&self) -> &StoreId {
        &self.id
    }

    pub fn brand_id(&self) -> &BrandId {
        &self.brand_id
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn currency_code(&self) -> &str {
        &self.currency_code
    }

    pub fn timezone(&self) -> &str {
        &self.timezone
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

    pub fn restore_as_active(
        &mut self,
        name: impl Into<String>,
        currency_code: impl Into<String>,
        timezone: impl Into<String>,
        now: Timestamp,
    ) -> Result<(), DomainError> {
        self.name = normalize_name(name)?;
        self.currency_code = normalize_currency_code(currency_code)?;
        self.timezone = normalize_timezone(timezone)?;
        self.status = OrganizationStatus::Active;
        self.updated_at = now;
        self.deleted_at = None;
        Ok(())
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

fn normalize_currency_code(value: impl Into<String>) -> Result<String, DomainError> {
    let value = value.into().trim().to_ascii_uppercase();
    if value.len() != 3 || !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Err(DomainError::InvalidCurrencyCode);
    }
    Ok(value)
}

fn normalize_timezone(value: impl Into<String>) -> Result<String, DomainError> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(DomainError::EmptyTimezone);
    }
    Ok(value)
}
