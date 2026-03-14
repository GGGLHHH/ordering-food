use crate::composition::{
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use crate::routes::menu::{self, MenuApiDoc};
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_identity_application as identity_application;
use ordering_food_menu_application::{
    ApplicationError as MenuApplicationError, Clock as MenuClock, CreateCategoryInput,
    CreateItemInput, CreateStoreInput, IdGenerator as MenuIdGenerator, MenuModule,
};
use ordering_food_menu_domain::{CategoryId, ItemId, MenuStatus, StoreId};
use ordering_food_menu_infrastructure_sqlx::build_menu_module;
use ordering_food_shared_kernel::Timestamp;
use std::sync::Arc;
use tracing::info;
use utoipa::OpenApi;
use uuid::Uuid;

pub fn register_menu() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "menu",
        depends_on: &[],
    };

    ApiContextRegistration::without_migration(descriptor, menu_bootstrap_registration)
}

fn menu_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = Arc::new(MenuClockAdapter {
            inner: platform.clock.clone(),
        });
        let id_generator = Arc::new(UuidV4MenuIdGenerator);
        async move {
            let module = build_menu_module(pg_pool, clock, id_generator);
            seed_menu_if_empty(&module)
                .await
                .map_err(|error| std::io::Error::other(error.to_string()))?;

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.add_route_mount(menu::MENU_ROUTE_PREFIX, menu::router(module.clone()));
            contribution.add_openapi_document(MenuApiDoc::openapi());
            contribution.retain_private(module);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}

async fn seed_menu_if_empty(module: &MenuModule) -> Result<(), MenuApplicationError> {
    if let Some(store) = module.store_queries.get_active().await? {
        info!(
            store_id = %store.store_id,
            slug = %store.slug,
            "menu seed skipped because an active store already exists"
        );
        return Ok(());
    }

    let store = module
        .create_store
        .execute(CreateStoreInput {
            slug: "ordering-food-demo".to_string(),
            name: "Ordering Food Demo Kitchen".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: MenuStatus::Active.as_str().to_string(),
        })
        .await?;

    let categories = [
        SeedCategory {
            slug: "featured",
            name: "Featured",
            description: Some("Popular picks for first-time visitors."),
            sort_order: 10,
        },
        SeedCategory {
            slug: "mains",
            name: "Mains",
            description: Some("Comfort food staples and filling bowls."),
            sort_order: 20,
        },
        SeedCategory {
            slug: "sides",
            name: "Sides",
            description: Some("Small plates that pair well with mains."),
            sort_order: 30,
        },
        SeedCategory {
            slug: "drinks",
            name: "Drinks",
            description: Some("Cold and hot drinks for the full meal."),
            sort_order: 40,
        },
    ];

    let mut category_ids = std::collections::BTreeMap::new();
    for category in categories {
        let created = module
            .create_category
            .execute(CreateCategoryInput {
                store_id: store.id().as_str().to_string(),
                slug: category.slug.to_string(),
                name: category.name.to_string(),
                description: category.description.map(ToOwned::to_owned),
                sort_order: category.sort_order,
                status: MenuStatus::Active.as_str().to_string(),
            })
            .await?;
        category_ids.insert(category.slug, created.id().as_str().to_string());
    }

    let items = [
        SeedItem {
            category_slug: "featured",
            slug: "crispy-chicken-bowl",
            name: "Crispy Chicken Bowl",
            description: Some("Golden chicken, soft egg, pickled greens, and rice."),
            image_url: None,
            price_amount: 3200,
            sort_order: 10,
        },
        SeedItem {
            category_slug: "featured",
            slug: "charcoal-beef-rice",
            name: "Charcoal Beef Rice",
            description: Some("Sliced beef over rice with scallion oil and sesame."),
            image_url: None,
            price_amount: 3600,
            sort_order: 20,
        },
        SeedItem {
            category_slug: "mains",
            slug: "soy-braised-pork-rice",
            name: "Soy Braised Pork Rice",
            description: Some("Slow-braised pork belly served over steamed rice."),
            image_url: None,
            price_amount: 2800,
            sort_order: 10,
        },
        SeedItem {
            category_slug: "mains",
            slug: "mushroom-noodle-soup",
            name: "Mushroom Noodle Soup",
            description: Some("Rich broth with mushrooms, greens, and springy noodles."),
            image_url: None,
            price_amount: 2600,
            sort_order: 20,
        },
        SeedItem {
            category_slug: "sides",
            slug: "garlic-cucumber-salad",
            name: "Garlic Cucumber Salad",
            description: Some("Chilled cucumber with garlic, vinegar, and sesame oil."),
            image_url: None,
            price_amount: 1200,
            sort_order: 10,
        },
        SeedItem {
            category_slug: "sides",
            slug: "seaweed-fries",
            name: "Seaweed Fries",
            description: Some("Crisp fries tossed with seaweed salt."),
            image_url: None,
            price_amount: 1500,
            sort_order: 20,
        },
        SeedItem {
            category_slug: "drinks",
            slug: "iced-lemon-tea",
            name: "Iced Lemon Tea",
            description: Some("Black tea with lemon and a clean citrus finish."),
            image_url: None,
            price_amount: 900,
            sort_order: 10,
        },
        SeedItem {
            category_slug: "drinks",
            slug: "jasmine-milk-tea",
            name: "Jasmine Milk Tea",
            description: Some("Floral jasmine tea with a creamy finish."),
            image_url: None,
            price_amount: 1400,
            sort_order: 20,
        },
    ];

    for item in items {
        let category_id = category_ids
            .get(item.category_slug)
            .expect("seed category should exist")
            .clone();
        module
            .create_item
            .execute(CreateItemInput {
                store_id: store.id().as_str().to_string(),
                category_id,
                slug: item.slug.to_string(),
                name: item.name.to_string(),
                description: item.description.map(ToOwned::to_owned),
                image_url: item.image_url.map(ToOwned::to_owned),
                price_amount: item.price_amount,
                sort_order: item.sort_order,
                status: MenuStatus::Active.as_str().to_string(),
            })
            .await?;
    }

    info!(
        store_id = %store.id().as_str(),
        "menu seed created default store, categories, and items"
    );
    Ok(())
}

struct SeedCategory {
    slug: &'static str,
    name: &'static str,
    description: Option<&'static str>,
    sort_order: i32,
}

struct SeedItem {
    category_slug: &'static str,
    slug: &'static str,
    name: &'static str,
    description: Option<&'static str>,
    image_url: Option<&'static str>,
    price_amount: i64,
    sort_order: i32,
}

struct MenuClockAdapter {
    inner: Arc<dyn identity_application::Clock>,
}

impl MenuClock for MenuClockAdapter {
    fn now(&self) -> Timestamp {
        self.inner.now()
    }
}

struct UuidV4MenuIdGenerator;

impl MenuIdGenerator for UuidV4MenuIdGenerator {
    fn next_store_id(&self) -> StoreId {
        StoreId::new(Uuid::new_v4().to_string())
    }

    fn next_category_id(&self) -> CategoryId {
        CategoryId::new(Uuid::new_v4().to_string())
    }

    fn next_item_id(&self) -> ItemId {
        ItemId::new(Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::seed_menu_if_empty;
    use async_trait::async_trait;
    use ordering_food_menu_application::{
        ApplicationError, CategoryReadModel, CategoryReadRepository, CategoryRepository, Clock,
        IdGenerator, ItemListFilter, ItemReadModel, ItemReadRepository, ItemRepository, MenuModule,
        StoreReadModel, StoreReadRepository, StoreRepository, TransactionContext,
        TransactionManager,
    };
    use ordering_food_menu_domain::{
        Category, CategoryId, Item, ItemId, MenuStatus, Store, StoreId,
    };
    use ordering_food_shared_kernel::Timestamp;
    use std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

    #[derive(Default)]
    struct FakeTransactionContext;

    impl TransactionContext for FakeTransactionContext {
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
            self
        }
    }

    #[derive(Default)]
    struct FakeTransactionManager;

    #[async_trait]
    impl TransactionManager for FakeTransactionManager {
        async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
            Ok(Box::new(FakeTransactionContext))
        }

        async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    struct FakeClock;

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            datetime!(2026-03-13 10:00 UTC)
        }
    }

    struct FakeIdGenerator {
        next_store: Mutex<u32>,
        next_category: Mutex<u32>,
        next_item: Mutex<u32>,
    }

    impl Default for FakeIdGenerator {
        fn default() -> Self {
            Self {
                next_store: Mutex::new(1),
                next_category: Mutex::new(1),
                next_item: Mutex::new(1),
            }
        }
    }

    impl IdGenerator for FakeIdGenerator {
        fn next_store_id(&self) -> StoreId {
            let mut next = self.next_store.lock().unwrap();
            let value = format!("00000000-0000-4000-8000-{:012}", *next);
            *next += 1;
            StoreId::new(value)
        }

        fn next_category_id(&self) -> CategoryId {
            let mut next = self.next_category.lock().unwrap();
            let value = format!("10000000-0000-4000-8000-{:012}", *next);
            *next += 1;
            CategoryId::new(value)
        }

        fn next_item_id(&self) -> ItemId {
            let mut next = self.next_item.lock().unwrap();
            let value = format!("20000000-0000-4000-8000-{:012}", *next);
            *next += 1;
            ItemId::new(value)
        }
    }

    #[derive(Default)]
    struct InMemoryState {
        stores: HashMap<String, Store>,
        categories: HashMap<String, Category>,
        items: HashMap<String, Item>,
    }

    #[derive(Clone, Default)]
    struct InMemoryMenuRepository {
        state: Arc<Mutex<InMemoryState>>,
    }

    #[async_trait]
    impl StoreRepository for InMemoryMenuRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            store_id: &StoreId,
        ) -> Result<Option<Store>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .stores
                .get(store_id.as_str())
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            store: &Store,
        ) -> Result<(), ApplicationError> {
            self.state
                .lock()
                .unwrap()
                .stores
                .insert(store.id().as_str().to_string(), store.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl CategoryRepository for InMemoryMenuRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            category_id: &CategoryId,
        ) -> Result<Option<Category>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .categories
                .get(category_id.as_str())
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            category: &Category,
        ) -> Result<(), ApplicationError> {
            self.state
                .lock()
                .unwrap()
                .categories
                .insert(category.id().as_str().to_string(), category.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl ItemRepository for InMemoryMenuRepository {
        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            item: &Item,
        ) -> Result<(), ApplicationError> {
            self.state
                .lock()
                .unwrap()
                .items
                .insert(item.id().as_str().to_string(), item.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl StoreReadRepository for InMemoryMenuRepository {
        async fn get_active(&self) -> Result<Option<StoreReadModel>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .stores
                .values()
                .filter(|store| store.status() == MenuStatus::Active && !store.is_deleted())
                .min_by_key(|store| store.created_at())
                .map(|store| StoreReadModel {
                    store_id: store.id().as_str().to_string(),
                    slug: store.slug().to_string(),
                    name: store.name().to_string(),
                    currency_code: store.currency_code().to_string(),
                    timezone: store.timezone().to_string(),
                    status: store.status().as_str().to_string(),
                    created_at: store.created_at(),
                    updated_at: store.updated_at(),
                    deleted_at: store.deleted_at(),
                }))
        }
    }

    #[async_trait]
    impl CategoryReadRepository for InMemoryMenuRepository {
        async fn list_active_by_store(
            &self,
            store_id: &StoreId,
        ) -> Result<Vec<CategoryReadModel>, ApplicationError> {
            let mut categories = self
                .state
                .lock()
                .unwrap()
                .categories
                .values()
                .filter(|category| {
                    category.store_id() == store_id
                        && category.status() == MenuStatus::Active
                        && !category.is_deleted()
                })
                .map(|category| CategoryReadModel {
                    category_id: category.id().as_str().to_string(),
                    store_id: category.store_id().as_str().to_string(),
                    slug: category.slug().to_string(),
                    name: category.name().to_string(),
                    description: category.description().map(ToOwned::to_owned),
                    sort_order: category.sort_order(),
                    status: category.status().as_str().to_string(),
                    created_at: category.created_at(),
                    updated_at: category.updated_at(),
                    deleted_at: category.deleted_at(),
                })
                .collect::<Vec<_>>();
            categories.sort_by(|left, right| {
                left.sort_order
                    .cmp(&right.sort_order)
                    .then_with(|| left.category_id.cmp(&right.category_id))
            });
            Ok(categories)
        }

        async fn get_active_by_slug(
            &self,
            store_id: &StoreId,
            slug: &str,
        ) -> Result<Option<CategoryReadModel>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .categories
                .values()
                .find(|category| {
                    category.store_id() == store_id
                        && category.slug() == slug
                        && category.status() == MenuStatus::Active
                        && !category.is_deleted()
                })
                .map(|category| CategoryReadModel {
                    category_id: category.id().as_str().to_string(),
                    store_id: category.store_id().as_str().to_string(),
                    slug: category.slug().to_string(),
                    name: category.name().to_string(),
                    description: category.description().map(ToOwned::to_owned),
                    sort_order: category.sort_order(),
                    status: category.status().as_str().to_string(),
                    created_at: category.created_at(),
                    updated_at: category.updated_at(),
                    deleted_at: category.deleted_at(),
                }))
        }
    }

    #[async_trait]
    impl ItemReadRepository for InMemoryMenuRepository {
        async fn list_active_by_store(
            &self,
            store_id: &StoreId,
            filter: ItemListFilter,
        ) -> Result<Vec<ItemReadModel>, ApplicationError> {
            let mut items = self
                .state
                .lock()
                .unwrap()
                .items
                .values()
                .filter(|item| {
                    item.store_id() == store_id
                        && item.status() == MenuStatus::Active
                        && !item.is_deleted()
                        && filter
                            .category_id
                            .as_ref()
                            .is_none_or(|category_id| item.category_id() == category_id)
                })
                .map(|item| ItemReadModel {
                    item_id: item.id().as_str().to_string(),
                    store_id: item.store_id().as_str().to_string(),
                    category_id: item.category_id().as_str().to_string(),
                    slug: item.slug().to_string(),
                    name: item.name().to_string(),
                    description: item.description().map(ToOwned::to_owned),
                    image_url: item.image_url().map(ToOwned::to_owned),
                    price_amount: item.price_amount(),
                    sort_order: item.sort_order(),
                    status: item.status().as_str().to_string(),
                    created_at: item.created_at(),
                    updated_at: item.updated_at(),
                    deleted_at: item.deleted_at(),
                })
                .collect::<Vec<_>>();
            items.sort_by(|left, right| {
                left.sort_order
                    .cmp(&right.sort_order)
                    .then_with(|| left.item_id.cmp(&right.item_id))
            });
            Ok(items)
        }

        async fn get_active_by_id(
            &self,
            item_id: &ItemId,
        ) -> Result<Option<ItemReadModel>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .items
                .get(item_id.as_str())
                .filter(|item| item.status() == MenuStatus::Active && !item.is_deleted())
                .map(|item| ItemReadModel {
                    item_id: item.id().as_str().to_string(),
                    store_id: item.store_id().as_str().to_string(),
                    category_id: item.category_id().as_str().to_string(),
                    slug: item.slug().to_string(),
                    name: item.name().to_string(),
                    description: item.description().map(ToOwned::to_owned),
                    image_url: item.image_url().map(ToOwned::to_owned),
                    price_amount: item.price_amount(),
                    sort_order: item.sort_order(),
                    status: item.status().as_str().to_string(),
                    created_at: item.created_at(),
                    updated_at: item.updated_at(),
                    deleted_at: item.deleted_at(),
                }))
        }
    }

    fn build_module(repository: Arc<InMemoryMenuRepository>) -> Arc<MenuModule> {
        Arc::new(MenuModule::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository,
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock),
            Arc::new(FakeIdGenerator::default()),
        ))
    }

    #[tokio::test]
    async fn seed_menu_if_empty_creates_default_store_categories_and_items() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        let module = build_module(repository.clone());

        seed_menu_if_empty(&module).await.unwrap();

        let state = repository.state.lock().unwrap();
        assert_eq!(state.stores.len(), 1);
        assert_eq!(state.categories.len(), 4);
        assert_eq!(state.items.len(), 8);
        assert!(
            state
                .stores
                .values()
                .any(|store| store.slug() == "ordering-food-demo")
        );
    }

    #[tokio::test]
    async fn seed_menu_if_empty_skips_when_active_store_exists() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        let existing_store = Store::create(
            StoreId::new("store-existing"),
            "existing-store",
            "Existing Store",
            "CNY",
            "Asia/Shanghai",
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
        )
        .unwrap();
        repository
            .state
            .lock()
            .unwrap()
            .stores
            .insert(existing_store.id().as_str().to_string(), existing_store);
        let module = build_module(repository.clone());

        seed_menu_if_empty(&module).await.unwrap();

        let state = repository.state.lock().unwrap();
        assert_eq!(state.stores.len(), 1);
        assert!(state.categories.is_empty());
        assert!(state.items.is_empty());
    }
}
