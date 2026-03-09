pub mod api;
pub mod health;

use crate::{
    app::AppState,
    readiness::DependencyChecks,
};
use ordering_food_shared::{
    error::{ErrorDetails, ErrorEnvelope, FieldIssue, FieldLocation},
    http::{self, PageMeta},
};
use axum::{Router, extract::DefaultBodyLimit};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) const API_PREFIX: &str = "/api";

#[derive(OpenApi)]
#[openapi(
    paths(
        health::live,
        health::ready,
    ),
    components(
        schemas(
            crate::readiness::LiveResponse,
            crate::readiness::ReadyResponse,
            DependencyChecks,
            ErrorEnvelope,
            ErrorDetails,
            FieldIssue,
            FieldLocation,
            PageMeta,
        )
    ),
    nest(
        (path = API_PREFIX, api = api::ApiGroupDoc),
        (path = API_PREFIX, api = ordering_food_user::http::UserApiDoc),
    ),
    tags(
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let health_router = health::router().method_not_allowed_fallback(http::method_not_allowed);
    let api_router = api::router()
        .merge(ordering_food_user::http::router())
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(DefaultBodyLimit::max(http::API_BODY_LIMIT_BYTES));

    Router::new()
        .nest("/health", health_router)
        .nest(API_PREFIX, api_router)
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .fallback(http::not_found)
        .with_state(state)
}
