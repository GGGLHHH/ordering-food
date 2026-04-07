use crate::{
    ApplicationError, AttachStoreCatalog, AttachStoreCatalogInput, BootstrapBrandCatalog,
    BootstrapBrandCatalogInput, BrandCatalogQueryService, CategoryQueryService, CreateCategory,
    CreateCategoryInput, CreateItem, CreateItemInput, ItemQueryService, StoreCatalogQueryService,
    UpsertStoreItemListing, UpsertStoreItemListingInput,
};
use ordering_food_organization_published::StoreSummary;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapDefaultCatalogInput {
    pub active_store: StoreSummary,
    pub brand_slug: String,
    pub brand_name: String,
    pub categories: Vec<BootstrapDefaultCategoryInput>,
    pub items: Vec<BootstrapDefaultItemInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapDefaultCategoryInput {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapDefaultItemInput {
    pub category_slug: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub price_amount: i64,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapDefaultCatalogOutcome {
    Skipped {
        store_id: String,
        brand_id: String,
    },
    Seeded {
        store_id: String,
        brand_id: String,
        category_count: usize,
        item_count: usize,
    },
}

pub struct BootstrapDefaultCatalog {
    bootstrap_brand_catalog: Arc<BootstrapBrandCatalog>,
    attach_store_catalog: Arc<AttachStoreCatalog>,
    create_category: Arc<CreateCategory>,
    create_item: Arc<CreateItem>,
    upsert_store_item_listing: Arc<UpsertStoreItemListing>,
    brand_catalog_queries: Arc<BrandCatalogQueryService>,
    store_catalog_queries: Arc<StoreCatalogQueryService>,
    category_queries: Arc<CategoryQueryService>,
    item_queries: Arc<ItemQueryService>,
}

impl BootstrapDefaultCatalog {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bootstrap_brand_catalog: Arc<BootstrapBrandCatalog>,
        attach_store_catalog: Arc<AttachStoreCatalog>,
        create_category: Arc<CreateCategory>,
        create_item: Arc<CreateItem>,
        upsert_store_item_listing: Arc<UpsertStoreItemListing>,
        brand_catalog_queries: Arc<BrandCatalogQueryService>,
        store_catalog_queries: Arc<StoreCatalogQueryService>,
        category_queries: Arc<CategoryQueryService>,
        item_queries: Arc<ItemQueryService>,
    ) -> Self {
        Self {
            bootstrap_brand_catalog,
            attach_store_catalog,
            create_category,
            create_item,
            upsert_store_item_listing,
            brand_catalog_queries,
            store_catalog_queries,
            category_queries,
            item_queries,
        }
    }

    pub async fn execute(
        &self,
        input: BootstrapDefaultCatalogInput,
    ) -> Result<BootstrapDefaultCatalogOutcome, ApplicationError> {
        let active_store = input.active_store;
        let brand_catalog_id = self
            .ensure_brand_catalog(&active_store, &input.brand_slug, &input.brand_name)
            .await?;
        let store_catalog_id = self.ensure_store_catalog(&active_store).await?;

        let mut category_ids = BTreeMap::new();
        for category in &input.categories {
            let category_id = self.ensure_category(&brand_catalog_id, category).await?;
            category_ids.insert(category.slug.clone(), category_id);
        }

        let item_count = input.items.len();
        for item in &input.items {
            let category_id = category_ids.get(&item.category_slug).ok_or_else(|| {
                ApplicationError::unexpected(format!(
                    "bootstrap category `{}` was not available for item `{}`",
                    item.category_slug, item.slug
                ))
            })?;
            let item_id = self
                .ensure_item(&brand_catalog_id, category_id, item)
                .await?;
            self.upsert_store_item_listing
                .execute(UpsertStoreItemListingInput {
                    store_catalog_id: store_catalog_id.clone(),
                    item_id,
                    price_amount: item.price_amount,
                    status: "sellable".to_string(),
                    display_rule: "listed".to_string(),
                })
                .await?;
        }

        Ok(BootstrapDefaultCatalogOutcome::Seeded {
            store_id: active_store.store_id,
            brand_id: active_store.brand_id,
            category_count: category_ids.len(),
            item_count,
        })
    }

    async fn ensure_brand_catalog(
        &self,
        active_store: &StoreSummary,
        brand_slug: &str,
        brand_name: &str,
    ) -> Result<String, ApplicationError> {
        let input = BootstrapBrandCatalogInput {
            brand_id: active_store.brand_id.clone(),
            slug: brand_slug.to_string(),
            name: brand_name.to_string(),
        };

        match self.bootstrap_brand_catalog.execute(input).await {
            Ok(brand_catalog_id) => Ok(brand_catalog_id),
            Err(error @ ApplicationError::Conflict { .. }) => self
                .brand_catalog_queries
                .find_by_brand_id(&active_store.brand_id)
                .await?
                .map(|catalog| catalog.brand_catalog_id)
                .ok_or(error),
            Err(error) => Err(error),
        }
    }

    async fn ensure_store_catalog(
        &self,
        active_store: &StoreSummary,
    ) -> Result<String, ApplicationError> {
        let input = AttachStoreCatalogInput {
            brand_id: active_store.brand_id.clone(),
            store_id: active_store.store_id.clone(),
        };

        match self.attach_store_catalog.execute(input).await {
            Ok(store_catalog_id) => Ok(store_catalog_id),
            Err(error @ ApplicationError::Conflict { .. }) => self
                .store_catalog_queries
                .find_by_store_id(&active_store.store_id)
                .await?
                .map(|catalog| catalog.store_catalog_id)
                .ok_or(error),
            Err(error) => Err(error),
        }
    }

    async fn ensure_category(
        &self,
        brand_catalog_id: &str,
        seed: &BootstrapDefaultCategoryInput,
    ) -> Result<String, ApplicationError> {
        match self
            .create_category
            .execute(CreateCategoryInput {
                brand_catalog_id: brand_catalog_id.to_string(),
                slug: seed.slug.clone(),
                name: seed.name.clone(),
                description: seed.description.clone(),
                sort_order: seed.sort_order,
            })
            .await
        {
            Ok(category_id) => Ok(category_id),
            Err(error @ ApplicationError::Conflict { .. }) => self
                .category_queries
                .find_by_slug(brand_catalog_id, &seed.slug)
                .await?
                .map(|category| category.category_id)
                .ok_or(error),
            Err(error) => Err(error),
        }
    }

    async fn ensure_item(
        &self,
        brand_catalog_id: &str,
        category_id: &str,
        seed: &BootstrapDefaultItemInput,
    ) -> Result<String, ApplicationError> {
        match self
            .create_item
            .execute(CreateItemInput {
                brand_catalog_id: brand_catalog_id.to_string(),
                category_id: category_id.to_string(),
                slug: seed.slug.clone(),
                name: seed.name.clone(),
                description: seed.description.clone(),
                image_url: seed.image_url.clone(),
                sort_order: seed.sort_order,
            })
            .await
        {
            Ok(item_id) => Ok(item_id),
            Err(error @ ApplicationError::Conflict { .. }) => self
                .item_queries
                .find_by_slug(brand_catalog_id, &seed.slug)
                .await?
                .map(|item| item.item_id)
                .ok_or(error),
            Err(error) => Err(error),
        }
    }
}
