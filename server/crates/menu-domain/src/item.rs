use crate::{CategoryId, DomainError, ItemId, MenuStatus, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    id: ItemId,
    store_id: StoreId,
    category_id: CategoryId,
    slug: String,
    name: String,
    description: Option<String>,
    image_url: Option<String>,
    price_amount: i64,
    sort_order: i32,
    status: MenuStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
    deleted_at: Option<Timestamp>,
}

impl Item {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: ItemId,
        store_id: StoreId,
        category_id: CategoryId,
        slug: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        image_url: Option<String>,
        price_amount: i64,
        sort_order: i32,
        status: MenuStatus,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        if price_amount < 0 {
            return Err(DomainError::NegativePriceAmount);
        }

        Ok(Self {
            id,
            store_id,
            category_id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            description: trim_option(description),
            image_url: trim_option(image_url),
            price_amount,
            sort_order,
            status,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: ItemId,
        store_id: StoreId,
        category_id: CategoryId,
        slug: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        image_url: Option<String>,
        price_amount: i64,
        sort_order: i32,
        status: MenuStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        if price_amount < 0 {
            return Err(DomainError::NegativePriceAmount);
        }

        Ok(Self {
            id,
            store_id,
            category_id,
            slug: normalize_slug(slug)?,
            name: normalize_name(name)?,
            description: trim_option(description),
            image_url: trim_option(image_url),
            price_amount,
            sort_order,
            status,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }

    pub fn store_id(&self) -> &StoreId {
        &self.store_id
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

    pub fn price_amount(&self) -> i64 {
        self.price_amount
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn item_rejects_negative_price() {
        let error = Item::create(
            ItemId::new("item-1"),
            StoreId::new("store-1"),
            CategoryId::new("category-1"),
            "dish",
            "Dish",
            None,
            None,
            -1,
            0,
            MenuStatus::Active,
            datetime!(2026-03-13 10:00 UTC),
        )
        .unwrap_err();

        assert_eq!(error, DomainError::NegativePriceAmount);
    }
}
