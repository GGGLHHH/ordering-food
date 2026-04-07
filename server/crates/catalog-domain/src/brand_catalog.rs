use crate::{BrandCatalogId, BrandId, DomainError};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandCatalog {
    id: BrandCatalogId,
    brand_id: BrandId,
    slug: String,
    name: String,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl BrandCatalog {
    pub fn create(
        id: BrandCatalogId,
        brand_id: BrandId,
        slug: impl Into<String>,
        name: impl Into<String>,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            brand_id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn id(&self) -> &BrandCatalogId {
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

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
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

#[cfg(test)]
mod tests {
    use super::BrandCatalog;
    use crate::{BrandCatalogId, BrandId};
    use time::macros::datetime;

    #[test]
    fn brand_catalog_normalizes_slug_and_name() {
        let catalog = BrandCatalog::create(
            BrandCatalogId::new("brand-catalog-1"),
            BrandId::new("brand-1"),
            " Demo-Catalog ",
            " Demo Catalog ",
            datetime!(2026-04-05 10:00 UTC),
        )
        .unwrap();

        assert_eq!(catalog.slug(), "demo-catalog");
        assert_eq!(catalog.name(), "Demo Catalog");
    }
}
