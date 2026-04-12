mod organization_scope_acl;

use ordering_food_catalog_application::{
    ApplicationError as CatalogApplicationError, BootstrapDefaultCatalogInput,
    BootstrapDefaultCatalogOutcome, BootstrapDefaultCategoryInput, BootstrapDefaultItemInput,
    CatalogModule, CatalogStoreScope, Clock as CatalogClock, IdGenerator as CatalogIdGenerator,
};
use ordering_food_catalog_domain::{BrandCatalogId, CategoryId, ItemId, StoreCatalogId};
use ordering_food_catalog_infrastructure_sqlx::build_catalog_sqlx_module;
use ordering_food_organization_published::{BrandLookupGateway, StoreScopeGateway, StoreSummary};
use organization_scope_acl::CatalogOrganizationScopeAclAdapter;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CatalogContextRuntime {
    module: Arc<CatalogModule>,
}

impl CatalogContextRuntime {
    pub fn module(&self) -> &Arc<CatalogModule> {
        &self.module
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogBootstrap {
    pub brand_slug: String,
    pub brand_name: String,
}

pub fn build_catalog_context_runtime(
    pg_pool: PgPool,
    brand_lookup_gateway: Arc<dyn BrandLookupGateway>,
    store_scope_gateway: Arc<dyn StoreScopeGateway>,
    clock: Arc<dyn CatalogClock>,
) -> CatalogContextRuntime {
    let organization_scope_acl = Arc::new(CatalogOrganizationScopeAclAdapter::new(
        brand_lookup_gateway,
        store_scope_gateway,
    ));
    let sqlx_module = build_catalog_sqlx_module(
        pg_pool,
        organization_scope_acl,
        clock,
        Arc::new(UuidV4CatalogIdGenerator),
    );
    CatalogContextRuntime {
        module: sqlx_module.application(),
    }
}

pub async fn seed_default_catalog(
    runtime: &CatalogContextRuntime,
    active_store: StoreSummary,
    bootstrap: CatalogBootstrap,
) -> Result<BootstrapDefaultCatalogOutcome, CatalogApplicationError> {
    let input = BootstrapDefaultCatalogInput {
        active_store: CatalogStoreScope {
            store_id: active_store.store_id,
            brand_id: active_store.brand_id,
            slug: active_store.slug,
            name: active_store.name,
            currency_code: active_store.currency_code,
            timezone: active_store.timezone,
            status: active_store.status,
        },
        brand_slug: bootstrap.brand_slug,
        brand_name: bootstrap.brand_name,
        categories: default_categories(),
        items: default_items(),
    };

    runtime
        .module
        .bootstrap_default_catalog()
        .execute(input)
        .await
}

pub mod projection {
    use ordering_food_catalog_published::{CatalogItemRef, CatalogPriceFact, StoreCatalogRef};

    pub trait CatalogProjectionUpdater {
        fn apply_store_catalog(&self, store_catalog: StoreCatalogRef);
        fn apply_catalog_item(&self, item: CatalogItemRef);
        fn apply_catalog_price(&self, price: CatalogPriceFact);
    }
}

struct UuidV4CatalogIdGenerator;

impl CatalogIdGenerator for UuidV4CatalogIdGenerator {
    fn next_brand_catalog_id(&self) -> BrandCatalogId {
        BrandCatalogId::new(Uuid::new_v4().to_string())
    }

    fn next_store_catalog_id(&self) -> StoreCatalogId {
        StoreCatalogId::new(Uuid::new_v4().to_string())
    }

    fn next_category_id(&self) -> CategoryId {
        CategoryId::new(Uuid::new_v4().to_string())
    }

    fn next_item_id(&self) -> ItemId {
        ItemId::new(Uuid::new_v4().to_string())
    }
}

fn default_categories() -> Vec<BootstrapDefaultCategoryInput> {
    vec![
        BootstrapDefaultCategoryInput {
            slug: "featured".to_string(),
            name: "Featured".to_string(),
            description: Some("Popular picks for first-time visitors.".to_string()),
            sort_order: 10,
        },
        BootstrapDefaultCategoryInput {
            slug: "mains".to_string(),
            name: "Mains".to_string(),
            description: Some("Comfort food staples and filling bowls.".to_string()),
            sort_order: 20,
        },
        BootstrapDefaultCategoryInput {
            slug: "sides".to_string(),
            name: "Sides".to_string(),
            description: Some("Small plates that pair well with mains.".to_string()),
            sort_order: 30,
        },
        BootstrapDefaultCategoryInput {
            slug: "drinks".to_string(),
            name: "Drinks".to_string(),
            description: Some("Cold and hot drinks for the full meal.".to_string()),
            sort_order: 40,
        },
    ]
}

fn default_items() -> Vec<BootstrapDefaultItemInput> {
    vec![
        BootstrapDefaultItemInput {
            category_slug: "featured".to_string(),
            slug: "crispy-chicken-bowl".to_string(),
            name: "Crispy Chicken Bowl".to_string(),
            description: Some("Golden chicken, soft egg, pickled greens, and rice.".to_string()),
            image_url: None,
            price_amount: 3200,
            sort_order: 10,
        },
        BootstrapDefaultItemInput {
            category_slug: "featured".to_string(),
            slug: "charcoal-beef-rice".to_string(),
            name: "Charcoal Beef Rice".to_string(),
            description: Some("Sliced beef over rice with scallion oil and sesame.".to_string()),
            image_url: None,
            price_amount: 3600,
            sort_order: 20,
        },
        BootstrapDefaultItemInput {
            category_slug: "mains".to_string(),
            slug: "soy-braised-pork-rice".to_string(),
            name: "Soy Braised Pork Rice".to_string(),
            description: Some("Slow-braised pork belly served over steamed rice.".to_string()),
            image_url: None,
            price_amount: 2800,
            sort_order: 10,
        },
        BootstrapDefaultItemInput {
            category_slug: "mains".to_string(),
            slug: "mushroom-noodle-soup".to_string(),
            name: "Mushroom Noodle Soup".to_string(),
            description: Some(
                "Rich broth with mushrooms, greens, and springy noodles.".to_string(),
            ),
            image_url: None,
            price_amount: 2600,
            sort_order: 20,
        },
        BootstrapDefaultItemInput {
            category_slug: "sides".to_string(),
            slug: "garlic-cucumber-salad".to_string(),
            name: "Garlic Cucumber Salad".to_string(),
            description: Some("Chilled cucumber with garlic, vinegar, and sesame oil.".to_string()),
            image_url: None,
            price_amount: 1200,
            sort_order: 10,
        },
        BootstrapDefaultItemInput {
            category_slug: "sides".to_string(),
            slug: "seaweed-fries".to_string(),
            name: "Seaweed Fries".to_string(),
            description: Some("Crisp fries tossed with seaweed salt.".to_string()),
            image_url: None,
            price_amount: 1500,
            sort_order: 20,
        },
        BootstrapDefaultItemInput {
            category_slug: "drinks".to_string(),
            slug: "iced-lemon-tea".to_string(),
            name: "Iced Lemon Tea".to_string(),
            description: Some("Black tea with lemon and a clean citrus finish.".to_string()),
            image_url: None,
            price_amount: 900,
            sort_order: 10,
        },
        BootstrapDefaultItemInput {
            category_slug: "drinks".to_string(),
            slug: "jasmine-milk-tea".to_string(),
            name: "Jasmine Milk Tea".to_string(),
            description: Some("Floral jasmine tea with a creamy finish.".to_string()),
            image_url: None,
            price_amount: 1400,
            sort_order: 20,
        },
    ]
}
