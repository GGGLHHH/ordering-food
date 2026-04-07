use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    http::{self, ApiPath, ApiQuery, RequestContext},
};
use axum::{Extension, Json, Router, routing::get};
use ordering_food_catalog_application::{
    ActiveCatalogContextReadModel, ApplicationError as CatalogApplicationError,
    CatalogItemListFilter, CatalogModule,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const CATALOG_ROUTE_PREFIX: &str = "/api/catalog";
pub(crate) const CATALOG_STORE_PATH: &str = "/api/catalog/store";
pub(crate) const CATALOG_CATEGORIES_PATH: &str = "/api/catalog/categories";
pub(crate) const CATALOG_ITEMS_PATH: &str = "/api/catalog/items";
pub(crate) const CATALOG_ITEM_PATH: &str = "/api/catalog/items/{item_id}";

const STORE_ROUTE_PATH: &str = "/store";
const CATEGORIES_ROUTE_PATH: &str = "/categories";
const ITEMS_ROUTE_PATH: &str = "/items";
const ITEM_ROUTE_PATH: &str = "/items/{item_id}";

pub fn router(module: Arc<CatalogModule>) -> Router<AppState> {
    Router::new()
        .route(STORE_ROUTE_PATH, get(get_store_catalog))
        .route(CATEGORIES_ROUTE_PATH, get(list_categories))
        .route(ITEMS_ROUTE_PATH, get(list_items))
        .route(ITEM_ROUTE_PATH, get(get_item))
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(Extension(module))
}

#[derive(OpenApi)]
#[openapi(
    paths(get_store_catalog, list_categories, list_items, get_item),
    components(
        schemas(
            ErrorEnvelope,
            CatalogStoreCatalogResponse,
            CatalogCategoryResponse,
            CatalogCategoriesResponse,
            CatalogItemsQuery,
            CatalogItemPath,
            CatalogItemResponse,
            CatalogItemsResponse,
        )
    ),
    tags(
        (name = "catalog", description = "Catalog read endpoints")
    )
)]
pub struct CatalogApiDoc;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CatalogStoreCatalogResponse {
    pub brand_catalog_id: String,
    pub store_catalog_id: String,
    pub brand_id: String,
    pub store_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
    pub display_rule: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CatalogCategoryResponse {
    pub category_id: String,
    pub brand_catalog_id: String,
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CatalogCategoriesResponse {
    pub categories: Vec<CatalogCategoryResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
pub struct CatalogItemsQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_slug: Option<String>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Path)]
pub struct CatalogItemPath {
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CatalogItemResponse {
    pub item_id: String,
    pub brand_catalog_id: String,
    pub store_catalog_id: String,
    pub category_id: String,
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub price_amount: i64,
    pub status: String,
    pub display_rule: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CatalogItemsResponse {
    pub items: Vec<CatalogItemResponse>,
}

#[utoipa::path(
    get,
    path = CATALOG_STORE_PATH,
    tag = "catalog",
    responses(
        (status = 200, description = "Fetch the active store catalog", body = CatalogStoreCatalogResponse),
        (status = 404, description = "Catalog store was not found", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_store_catalog(
    Extension(module): Extension<Arc<CatalogModule>>,
    context: RequestContext,
) -> Result<Json<CatalogStoreCatalogResponse>, AppError> {
    let active = module
        .active_catalog_queries()
        .load_active()
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?;
    Ok(Json(map_store_catalog_response(active)))
}

#[utoipa::path(
    get,
    path = CATALOG_CATEGORIES_PATH,
    tag = "catalog",
    responses(
        (status = 200, description = "List categories for the active brand catalog", body = CatalogCategoriesResponse),
        (status = 404, description = "Catalog store was not found", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn list_categories(
    Extension(module): Extension<Arc<CatalogModule>>,
    context: RequestContext,
) -> Result<Json<CatalogCategoriesResponse>, AppError> {
    let active = module
        .active_catalog_queries()
        .load_active()
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?;
    let categories = module
        .category_queries()
        .list_by_brand_catalog_id(&active.brand_catalog.brand_catalog_id)
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?
        .into_iter()
        .map(map_category_read_model)
        .collect();

    Ok(Json(CatalogCategoriesResponse { categories }))
}

#[utoipa::path(
    get,
    path = CATALOG_ITEMS_PATH,
    tag = "catalog",
    params(CatalogItemsQuery),
    responses(
        (status = 200, description = "List catalog items for the active store catalog", body = CatalogItemsResponse),
        (status = 404, description = "Catalog store was not found", body = ErrorEnvelope),
        (status = 422, description = "Query validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn list_items(
    Extension(module): Extension<Arc<CatalogModule>>,
    context: RequestContext,
    ApiQuery(query): ApiQuery<CatalogItemsQuery>,
) -> Result<Json<CatalogItemsResponse>, AppError> {
    let active = module
        .active_catalog_queries()
        .load_active()
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?;
    let category_id = resolve_category_filter(
        &module,
        &active.brand_catalog.brand_catalog_id,
        &query,
        context.request_id.clone(),
    )
    .await?;
    let items = module
        .item_queries()
        .list_by_brand_catalog_id(
            &active.brand_catalog.brand_catalog_id,
            CatalogItemListFilter { category_id },
        )
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?;
    let listings = module
        .store_item_listing_queries()
        .list_by_store_catalog_id(&active.store_catalog.store_catalog_id)
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?
        .into_iter()
        .map(|listing| (listing.item_id.clone(), listing))
        .collect::<BTreeMap<_, _>>();

    let items = items
        .into_iter()
        .filter_map(|item| {
            listings.get(&item.item_id).cloned().map(|listing| {
                map_item_read_model(&active.store_catalog.store_catalog_id, item, listing)
            })
        })
        .collect();

    Ok(Json(CatalogItemsResponse { items }))
}

#[utoipa::path(
    get,
    path = CATALOG_ITEM_PATH,
    tag = "catalog",
    params(CatalogItemPath),
    responses(
        (status = 200, description = "Fetch a catalog item for the active store catalog", body = CatalogItemResponse),
        (status = 404, description = "Catalog item was not found", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_item(
    Extension(module): Extension<Arc<CatalogModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<CatalogItemPath>,
) -> Result<Json<CatalogItemResponse>, AppError> {
    let active = module
        .active_catalog_queries()
        .load_active()
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?;
    let item = module
        .item_queries()
        .find_by_id(&path.item_id)
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?
        .filter(|item| item.brand_catalog_id == active.brand_catalog.brand_catalog_id)
        .ok_or_else(|| {
            AppError::not_found("catalog item was not found")
                .with_request_id(context.request_id.clone())
        })?;
    let listing = module
        .store_item_listing_queries()
        .find_by_item_id(
            &active.store_catalog.store_catalog_id,
            &path.item_id,
        )
        .await
        .map_err(|error| map_catalog_error(error, context.request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("catalog item was not found").with_request_id(context.request_id)
        })?;

    Ok(Json(map_item_read_model(
        &active.store_catalog.store_catalog_id,
        item,
        listing,
    )))
}

async fn resolve_category_filter(
    module: &CatalogModule,
    brand_catalog_id: &str,
    query: &CatalogItemsQuery,
    request_id: Option<String>,
) -> Result<Option<String>, AppError> {
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
        (Some(category_id), None) => Ok(Some(category_id.to_string())),
        (None, Some(category_slug)) => {
            let category = module
                .category_queries()
                .find_by_slug(brand_catalog_id, category_slug)
                .await
                .map_err(|error| map_catalog_error(error, request_id.clone()))?;
            Ok(category.map(|category| category.category_id))
        }
        (Some(category_id), Some(category_slug)) => {
            let category = module
                .category_queries()
                .find_by_slug(brand_catalog_id, category_slug)
                .await
                .map_err(|error| map_catalog_error(error, request_id.clone()))?
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
            Ok(Some(category_id.to_string()))
        }
    }
}

fn map_store_catalog_response(
    active: ActiveCatalogContextReadModel,
) -> CatalogStoreCatalogResponse {
    CatalogStoreCatalogResponse {
        brand_catalog_id: active.brand_catalog.brand_catalog_id,
        store_catalog_id: active.store_catalog.store_catalog_id,
        brand_id: active.store.brand_id,
        store_id: active.store.store_id,
        slug: active.store.slug,
        name: active.store.name,
        currency_code: active.store.currency_code,
        timezone: active.store.timezone,
        status: active.store_catalog.status,
        display_rule: active.store_catalog.display_rule,
    }
}

fn map_category_read_model(
    category: ordering_food_catalog_application::CategoryReadModel,
) -> CatalogCategoryResponse {
    CatalogCategoryResponse {
        category_id: category.category_id,
        brand_catalog_id: category.brand_catalog_id,
        slug: category.slug,
        name: category.name,
        description: category.description,
        sort_order: category.sort_order,
    }
}

fn map_item_read_model(
    store_catalog_id: &str,
    item: ordering_food_catalog_application::ItemReadModel,
    listing: ordering_food_catalog_application::StoreItemListingReadModel,
) -> CatalogItemResponse {
    CatalogItemResponse {
        item_id: item.item_id,
        brand_catalog_id: item.brand_catalog_id,
        store_catalog_id: store_catalog_id.to_string(),
        category_id: item.category_id,
        slug: item.slug,
        name: item.name,
        description: item.description,
        image_url: item.image_url,
        price_amount: listing.price_amount,
        status: listing.status,
        display_rule: listing.display_rule,
        sort_order: item.sort_order,
    }
}

fn map_catalog_error(error: CatalogApplicationError, request_id: Option<String>) -> AppError {
    match error {
        CatalogApplicationError::Validation { message } => {
            AppError::validation_error(message).with_request_id(request_id)
        }
        CatalogApplicationError::NotFound { message } => {
            AppError::not_found(message).with_request_id(request_id)
        }
        CatalogApplicationError::Conflict { message } => {
            AppError::conflict(message).with_request_id(request_id)
        }
        CatalogApplicationError::Unexpected { message, source } => match source {
            Some(source) => {
                AppError::internal(format!("{message}: {source}")).with_request_id(request_id)
            }
            None => AppError::internal(message).with_request_id(request_id),
        },
    }
}
