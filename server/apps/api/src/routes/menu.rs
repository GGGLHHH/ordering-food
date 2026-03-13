use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    http::{self, ApiPath, ApiQuery, RequestContext},
};
use axum::{Extension, Json, Router, routing::get};
use ordering_food_menu_application::{ApplicationError, ItemListFilter, MenuModule};
use ordering_food_menu_domain::{CategoryId, ItemId, StoreId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const MENU_ROUTE_PREFIX: &str = "/api/menu";
pub(crate) const MENU_STORE_PATH: &str = "/api/menu/store";
pub(crate) const MENU_CATEGORIES_PATH: &str = "/api/menu/categories";
pub(crate) const MENU_ITEMS_PATH: &str = "/api/menu/items";
pub(crate) const MENU_ITEM_PATH: &str = "/api/menu/items/{item_id}";

const STORE_ROUTE_PATH: &str = "/store";
const CATEGORIES_ROUTE_PATH: &str = "/categories";
const ITEMS_ROUTE_PATH: &str = "/items";
const ITEM_ROUTE_PATH: &str = "/items/{item_id}";

pub fn router(module: Arc<MenuModule>) -> Router<AppState> {
    Router::new()
        .route(STORE_ROUTE_PATH, get(get_store))
        .route(CATEGORIES_ROUTE_PATH, get(list_categories))
        .route(ITEMS_ROUTE_PATH, get(list_items))
        .route(ITEM_ROUTE_PATH, get(get_item))
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(Extension(module))
}

#[derive(OpenApi)]
#[openapi(
    paths(get_store, list_categories, list_items, get_item),
    components(
        schemas(
            ErrorEnvelope,
            MenuStoreResponse,
            MenuCategoryResponse,
            MenuCategoriesResponse,
            MenuItemsQuery,
            MenuItemPath,
            MenuItemResponse,
            MenuItemsResponse,
        )
    ),
    tags(
        (name = "menu", description = "Menu read endpoints")
    )
)]
pub struct MenuApiDoc;

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct MenuStoreResponse {
    pub store_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct MenuCategoryResponse {
    pub category_id: String,
    pub store_id: String,
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct MenuCategoriesResponse {
    pub categories: Vec<MenuCategoryResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema, TS)]
pub struct MenuItemsQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category_slug: Option<String>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema, TS)]
#[into_params(parameter_in = Path)]
pub struct MenuItemPath {
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct MenuItemResponse {
    pub item_id: String,
    pub store_id: String,
    pub category_id: String,
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub image_url: Option<String>,
    pub price_amount: i64,
    pub sort_order: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct MenuItemsResponse {
    pub items: Vec<MenuItemResponse>,
}

#[utoipa::path(
    get,
    path = MENU_STORE_PATH,
    tag = "menu",
    responses(
        (status = 200, description = "Fetch the active store", body = MenuStoreResponse),
        (status = 404, description = "Menu store was not found", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_store(
    Extension(module): Extension<Arc<MenuModule>>,
    context: RequestContext,
) -> Result<Json<MenuStoreResponse>, AppError> {
    let store = load_active_store(&module, context.request_id.clone()).await?;
    Ok(Json(store))
}

#[utoipa::path(
    get,
    path = MENU_CATEGORIES_PATH,
    tag = "menu",
    responses(
        (status = 200, description = "List active categories for the active store", body = MenuCategoriesResponse),
        (status = 404, description = "Menu store was not found", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn list_categories(
    Extension(module): Extension<Arc<MenuModule>>,
    context: RequestContext,
) -> Result<Json<MenuCategoriesResponse>, AppError> {
    let store = load_active_store(&module, context.request_id.clone()).await?;
    let categories = module
        .category_queries
        .list_active_by_store(&StoreId::new(store.store_id.clone()))
        .await
        .map_err(|error| map_menu_error(error, context.request_id.clone()))?
        .into_iter()
        .map(map_category_read_model)
        .collect();

    Ok(Json(MenuCategoriesResponse { categories }))
}

#[utoipa::path(
    get,
    path = MENU_ITEMS_PATH,
    tag = "menu",
    params(MenuItemsQuery),
    responses(
        (status = 200, description = "List active items for the active store", body = MenuItemsResponse),
        (status = 404, description = "Menu store was not found", body = ErrorEnvelope),
        (status = 422, description = "Query validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn list_items(
    Extension(module): Extension<Arc<MenuModule>>,
    context: RequestContext,
    ApiQuery(query): ApiQuery<MenuItemsQuery>,
) -> Result<Json<MenuItemsResponse>, AppError> {
    let store = load_active_store(&module, context.request_id.clone()).await?;
    let category_id =
        resolve_category_filter(&module, &store.store_id, &query, context.request_id.clone())
            .await?;
    let items = module
        .item_queries
        .list_active_by_store(
            &StoreId::new(store.store_id.clone()),
            ItemListFilter { category_id },
        )
        .await
        .map_err(|error| map_menu_error(error, context.request_id.clone()))?
        .into_iter()
        .map(map_item_read_model)
        .collect();

    Ok(Json(MenuItemsResponse { items }))
}

#[utoipa::path(
    get,
    path = MENU_ITEM_PATH,
    tag = "menu",
    params(MenuItemPath),
    responses(
        (status = 200, description = "Fetch an active item", body = MenuItemResponse),
        (status = 404, description = "Menu item was not found", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_item(
    Extension(module): Extension<Arc<MenuModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<MenuItemPath>,
) -> Result<Json<MenuItemResponse>, AppError> {
    let store = load_active_store(&module, context.request_id.clone()).await?;
    let item = module
        .item_queries
        .get_active_by_id(&ItemId::new(path.item_id))
        .await
        .map_err(|error| map_menu_error(error, context.request_id.clone()))?
        .filter(|item| item.store_id == store.store_id)
        .ok_or_else(|| {
            AppError::not_found("menu item was not found").with_request_id(context.request_id)
        })?;

    Ok(Json(map_item_read_model(item)))
}

async fn load_active_store(
    module: &MenuModule,
    request_id: Option<String>,
) -> Result<MenuStoreResponse, AppError> {
    let store = module
        .store_queries
        .get_active()
        .await
        .map_err(|error| map_menu_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("menu store was not found").with_request_id(request_id.clone())
        })?;

    Ok(MenuStoreResponse {
        store_id: store.store_id,
        slug: store.slug,
        name: store.name,
        currency_code: store.currency_code,
        timezone: store.timezone,
        status: store.status,
    })
}

async fn resolve_category_filter(
    module: &MenuModule,
    store_id: &str,
    query: &MenuItemsQuery,
    request_id: Option<String>,
) -> Result<Option<CategoryId>, AppError> {
    let category_id = query
        .category_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let category_slug = query
        .category_slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (category_id, category_slug) {
        (None, None) => Ok(None),
        (Some(category_id), None) => Ok(Some(CategoryId::new(category_id.to_string()))),
        (None, Some(category_slug)) => {
            let category = module
                .category_queries
                .get_active_by_slug(&StoreId::new(store_id.to_string()), category_slug)
                .await
                .map_err(|error| map_menu_error(error, request_id.clone()))?;
            Ok(category.map(|category| CategoryId::new(category.category_id)))
        }
        (Some(category_id), Some(category_slug)) => {
            let category = module
                .category_queries
                .get_active_by_slug(&StoreId::new(store_id.to_string()), category_slug)
                .await
                .map_err(|error| map_menu_error(error, request_id.clone()))?
                .ok_or_else(|| {
                    AppError::validation_error(
                        "category_id and category_slug must refer to the same category",
                    )
                    .with_request_id(request_id.clone())
                })?;
            if category.category_id != category_id {
                return Err(AppError::validation_error(
                    "category_id and category_slug must refer to the same category",
                )
                .with_request_id(request_id));
            }
            Ok(Some(CategoryId::new(category_id.to_string())))
        }
    }
}

fn map_category_read_model(
    category: ordering_food_menu_application::CategoryReadModel,
) -> MenuCategoryResponse {
    MenuCategoryResponse {
        category_id: category.category_id,
        store_id: category.store_id,
        slug: category.slug,
        name: category.name,
        description: category.description,
        sort_order: category.sort_order,
        status: category.status,
    }
}

fn map_item_read_model(item: ordering_food_menu_application::ItemReadModel) -> MenuItemResponse {
    MenuItemResponse {
        item_id: item.item_id,
        store_id: item.store_id,
        category_id: item.category_id,
        slug: item.slug,
        name: item.name,
        description: item.description,
        image_url: item.image_url,
        price_amount: item.price_amount,
        sort_order: item.sort_order,
        status: item.status,
    }
}

fn map_menu_error(error: ApplicationError, request_id: Option<String>) -> AppError {
    match error {
        ApplicationError::Validation { message } => {
            AppError::validation_error(message).with_request_id(request_id)
        }
        ApplicationError::NotFound { message } => {
            AppError::not_found(message).with_request_id(request_id)
        }
        ApplicationError::Conflict { message } => {
            AppError::conflict(message).with_request_id(request_id)
        }
        ApplicationError::Unexpected { .. } => {
            AppError::internal("internal server error").with_request_id(request_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppState,
        readiness::{DependencyChecks, ReadinessProbe},
    };
    use async_trait::async_trait;
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{HeaderName, Request, StatusCode},
        response::Response,
    };
    use ordering_food_menu_application::{
        CategoryReadModel, CategoryReadRepository, CategoryRepository, Clock, IdGenerator,
        ItemReadModel, ItemReadRepository, ItemRepository, MenuModule, StoreReadModel,
        StoreReadRepository, StoreRepository, TransactionContext, TransactionManager,
    };
    use ordering_food_menu_domain::{
        Category, CategoryId, Item, ItemId, MenuStatus, Store, StoreId,
    };
    use ordering_food_shared_kernel::Timestamp;
    use serde_json::Value;
    use std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;
    use tower::ServiceExt;
    use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

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

    struct FakeClock {
        now: Timestamp,
    }

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FakeIdGenerator;

    impl IdGenerator for FakeIdGenerator {
        fn next_store_id(&self) -> StoreId {
            StoreId::new("generated-store")
        }

        fn next_category_id(&self) -> CategoryId {
            CategoryId::new("generated-category")
        }

        fn next_item_id(&self) -> ItemId {
            ItemId::new("generated-item")
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

    impl InMemoryMenuRepository {
        fn seed_store(&self, store: Store) {
            self.state
                .lock()
                .unwrap()
                .stores
                .insert(store.id().as_str().to_string(), store);
        }

        fn seed_category(&self, category: Category) {
            self.state
                .lock()
                .unwrap()
                .categories
                .insert(category.id().as_str().to_string(), category);
        }

        fn seed_item(&self, item: Item) {
            self.state
                .lock()
                .unwrap()
                .items
                .insert(item.id().as_str().to_string(), item);
        }
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
            self.seed_store(store.clone());
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
            self.seed_category(category.clone());
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
            self.seed_item(item.clone());
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

    struct StubReadiness;

    #[async_trait]
    impl ReadinessProbe for StubReadiness {
        async fn check(&self) -> Result<DependencyChecks, AppError> {
            Ok(DependencyChecks::ok("ok", "ok"))
        }
    }

    fn build_test_app(repository: Arc<InMemoryMenuRepository>) -> Router {
        let module = Arc::new(MenuModule::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository,
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-03-13 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        ));
        let request_id_header = HeaderName::from_static("x-request-id");

        Router::new()
            .nest(MENU_ROUTE_PREFIX, router(module))
            .fallback(http::not_found)
            .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
            .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
            .with_state(AppState::new(Arc::new(StubReadiness)))
    }

    async fn response_json(response: Response<Body>) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn make_store(
        store_id: &str,
        slug: &str,
        status: MenuStatus,
        created_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Store {
        Store::rehydrate(
            StoreId::new(store_id),
            slug,
            slug,
            "CNY",
            "Asia/Shanghai",
            status,
            created_at,
            created_at,
            deleted_at,
        )
        .unwrap()
    }

    fn make_category(
        category_id: &str,
        store_id: &str,
        slug: &str,
        sort_order: i32,
        status: MenuStatus,
        created_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Category {
        Category::rehydrate(
            CategoryId::new(category_id),
            StoreId::new(store_id),
            slug,
            slug,
            None,
            sort_order,
            status,
            created_at,
            created_at,
            deleted_at,
        )
        .unwrap()
    }

    fn make_item(
        item_id: &str,
        store_id: &str,
        category_id: &str,
        slug: &str,
        sort_order: i32,
        status: MenuStatus,
        created_at: Timestamp,
        deleted_at: Option<Timestamp>,
    ) -> Item {
        Item::rehydrate(
            ItemId::new(item_id),
            StoreId::new(store_id),
            CategoryId::new(category_id),
            slug,
            slug,
            None,
            None,
            1200,
            sort_order,
            status,
            created_at,
            created_at,
            deleted_at,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn get_store_returns_active_store_payload() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        repository.seed_store(make_store(
            "store-1",
            "demo-store",
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
            None,
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(MENU_STORE_PATH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["store_id"], "store-1");
        assert_eq!(body["slug"], "demo-store");
        assert_eq!(body["currency_code"], "CNY");
    }

    #[tokio::test]
    async fn get_store_returns_not_found_when_no_active_store_exists() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        repository.seed_store(make_store(
            "store-1",
            "inactive-store",
            MenuStatus::Inactive,
            datetime!(2026-03-13 09:00 UTC),
            None,
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(MENU_STORE_PATH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_json(response).await;
        assert_eq!(body["code"], "not_found");
        assert_eq!(body["message"], "menu store was not found");
    }

    #[tokio::test]
    async fn list_categories_returns_sorted_active_categories_only() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        repository.seed_store(make_store(
            "store-1",
            "demo-store",
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-2",
            "store-1",
            "later",
            20,
            MenuStatus::Active,
            datetime!(2026-03-13 09:10 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-1",
            "store-1",
            "featured",
            10,
            MenuStatus::Active,
            datetime!(2026-03-13 09:05 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-3",
            "store-1",
            "hidden",
            5,
            MenuStatus::Inactive,
            datetime!(2026-03-13 09:01 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-4",
            "store-1",
            "deleted",
            1,
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
            Some(datetime!(2026-03-13 09:30 UTC)),
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(MENU_CATEGORIES_PATH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["categories"].as_array().unwrap().len(), 2);
        assert_eq!(body["categories"][0]["slug"], "featured");
        assert_eq!(body["categories"][1]["slug"], "later");
    }

    #[tokio::test]
    async fn list_items_filters_by_category_slug() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        repository.seed_store(make_store(
            "store-1",
            "demo-store",
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-featured",
            "store-1",
            "featured",
            10,
            MenuStatus::Active,
            datetime!(2026-03-13 09:05 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-mains",
            "store-1",
            "mains",
            20,
            MenuStatus::Active,
            datetime!(2026-03-13 09:10 UTC),
            None,
        ));
        repository.seed_item(make_item(
            "item-2",
            "store-1",
            "category-featured",
            "second",
            20,
            MenuStatus::Active,
            datetime!(2026-03-13 09:20 UTC),
            None,
        ));
        repository.seed_item(make_item(
            "item-1",
            "store-1",
            "category-featured",
            "first",
            10,
            MenuStatus::Active,
            datetime!(2026-03-13 09:15 UTC),
            None,
        ));
        repository.seed_item(make_item(
            "item-3",
            "store-1",
            "category-mains",
            "third",
            5,
            MenuStatus::Active,
            datetime!(2026-03-13 09:12 UTC),
            None,
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/menu/items?category_slug=featured")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["items"].as_array().unwrap().len(), 2);
        assert_eq!(body["items"][0]["slug"], "first");
        assert_eq!(body["items"][1]["slug"], "second");
    }

    #[tokio::test]
    async fn list_items_returns_validation_error_when_category_id_and_slug_disagree() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        repository.seed_store(make_store(
            "store-1",
            "demo-store",
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-featured",
            "store-1",
            "featured",
            10,
            MenuStatus::Active,
            datetime!(2026-03-13 09:05 UTC),
            None,
        ));
        repository.seed_category(make_category(
            "category-mains",
            "store-1",
            "mains",
            20,
            MenuStatus::Active,
            datetime!(2026-03-13 09:10 UTC),
            None,
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/menu/items?category_id=category-mains&category_slug=featured")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_json(response).await;
        assert_eq!(body["code"], "validation_error");
        assert_eq!(
            body["message"],
            "category_id and category_slug must refer to the same category"
        );
    }

    #[tokio::test]
    async fn get_item_returns_not_found_when_item_belongs_to_another_store() {
        let repository = Arc::new(InMemoryMenuRepository::default());
        repository.seed_store(make_store(
            "store-1",
            "demo-store",
            MenuStatus::Active,
            datetime!(2026-03-13 09:00 UTC),
            None,
        ));
        repository.seed_store(make_store(
            "store-2",
            "other-store",
            MenuStatus::Active,
            datetime!(2026-03-13 10:00 UTC),
            None,
        ));
        repository.seed_item(make_item(
            "item-1",
            "store-2",
            "category-2",
            "other-item",
            10,
            MenuStatus::Active,
            datetime!(2026-03-13 10:10 UTC),
            None,
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/menu/items/item-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_json(response).await;
        assert_eq!(body["code"], "not_found");
        assert_eq!(body["message"], "menu item was not found");
    }
}
