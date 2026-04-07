use crate::{BrandCatalogId, CategoryId, DomainError, ItemId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    id: ItemId,
    brand_catalog_id: BrandCatalogId,
    category_id: CategoryId,
    slug: String,
    name: String,
    description: Option<String>,
    image_url: Option<String>,
    sort_order: i32,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Item {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: ItemId,
        brand_catalog_id: BrandCatalogId,
        category_id: CategoryId,
        slug: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        image_url: Option<String>,
        sort_order: i32,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            brand_catalog_id,
            category_id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            description: trim_option(description),
            image_url: trim_option(image_url),
            sort_order,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }

    pub fn brand_catalog_id(&self) -> &BrandCatalogId {
        &self.brand_catalog_id
    }

    pub fn category_id(&self) -> &CategoryId {
        &self.category_id
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

    pub fn image_url(&self) -> Option<&str> {
        self.image_url.as_deref()
    }

    pub fn sort_order(&self) -> i32 {
        self.sort_order
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
