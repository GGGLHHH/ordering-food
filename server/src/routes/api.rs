use crate::app::AppState;
use axum::{
    Json, Router,
    routing::{get, post},
};
use ordering_food_shared::{
    error::{ErrorDetails, ErrorEnvelope, FieldIssue, FieldLocation},
    http::{ApiJson, ApiPath, ApiQuery, PageMeta},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const EXAMPLE_ECHO_PATH: &str = "/examples/echo";
pub(crate) const EXAMPLE_SEARCH_PATH: &str = "/examples/search";
pub(crate) const EXAMPLE_ITEM_PATH: &str = "/examples/items/{item_id}";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(EXAMPLE_ECHO_PATH, post(echo_payload))
        .route(EXAMPLE_SEARCH_PATH, get(search_examples))
        .route(EXAMPLE_ITEM_PATH, get(get_example_item))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        echo_payload,
        search_examples,
        get_example_item,
    ),
    components(
        schemas(
            ErrorEnvelope,
            ErrorDetails,
            FieldIssue,
            FieldLocation,
            PageMeta,
            ExamplePayload,
            ExamplePayloadResponse,
            ExampleSearchResponse,
            ExampleItemResponse,
        )
    ),
    tags(
        (name = "examples", description = "HTTP contract example endpoints")
    )
)]
pub struct ApiGroupDoc;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExamplePayload {
    pub name: String,
    pub quantity: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExamplePayloadResponse {
    pub accepted: bool,
    pub payload: ExamplePayload,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ExampleSearchQuery {
    pub page: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExampleSearchResponse {
    pub page: u32,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ExampleItemPath {
    pub item_id: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExampleItemResponse {
    pub item_id: u64,
}

#[utoipa::path(
    post,
    path = EXAMPLE_ECHO_PATH,
    tag = "examples",
    request_body = ExamplePayload,
    responses(
        (status = 200, description = "Echo JSON payload", body = ExamplePayloadResponse),
        (status = 400, description = "Invalid request body", body = ErrorEnvelope),
        (status = 413, description = "Request body exceeds limit", body = ErrorEnvelope),
        (status = 415, description = "Unsupported media type", body = ErrorEnvelope),
        (status = 422, description = "JSON validation error", body = ErrorEnvelope)
    )
)]
pub async fn echo_payload(
    ApiJson(payload): ApiJson<ExamplePayload>,
) -> Json<ExamplePayloadResponse> {
    Json(ExamplePayloadResponse {
        accepted: true,
        payload,
    })
}

#[utoipa::path(
    get,
    path = EXAMPLE_SEARCH_PATH,
    tag = "examples",
    params(ExampleSearchQuery),
    responses(
        (status = 200, description = "Echo parsed query", body = ExampleSearchResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorEnvelope)
    )
)]
pub async fn search_examples(
    ApiQuery(query): ApiQuery<ExampleSearchQuery>,
) -> Json<ExampleSearchResponse> {
    Json(ExampleSearchResponse { page: query.page })
}

#[utoipa::path(
    get,
    path = EXAMPLE_ITEM_PATH,
    tag = "examples",
    params(ExampleItemPath),
    responses(
        (status = 200, description = "Echo parsed path", body = ExampleItemResponse),
        (status = 400, description = "Invalid path parameters", body = ErrorEnvelope)
    )
)]
pub async fn get_example_item(
    ApiPath(path): ApiPath<ExampleItemPath>,
) -> Json<ExampleItemResponse> {
    Json(ExampleItemResponse {
        item_id: path.item_id,
    })
}
