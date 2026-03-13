use crate::{DomainError, MenuStatus, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    id: StoreId,
    slug: String,
    name: String,
    currency_code: String,
    timezone: String,
    status: MenuStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
    deleted_at: Option<Timestamp>,
}

impl Store {
    pub fn create(
        id: StoreId,
        slug: impl Into<String>,
        name: impl Into<String>,
        currency_code: impl Into<String>,
        timezone: impl Into<String>,
        status: MenuStatus,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            currency_code: normalize_currency_code(currency_code)?,
            timezone: normalize_timezone(timezone)?,
            status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    pub fn rehydrate(
        id: StoreId,
        slug: impl Into<String>,
        name: impl Into<String>,
        currency_code: impl Into<String>,
        timezone: impl Into<String>,
        status: MenuStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            slug: normalize_slug(slug)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn store_rejects_invalid_currency_code() {
        let error = Store::create(
            StoreId::new("store-1"),
            "demo",
            "Demo",
            "rmbb",
            "Asia/Shanghai",
            MenuStatus::Active,
            datetime!(2026-03-13 10:00 UTC),
        )
        .unwrap_err();

        assert_eq!(error, DomainError::InvalidCurrencyCode);
    }
}
