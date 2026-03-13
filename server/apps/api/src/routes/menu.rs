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
