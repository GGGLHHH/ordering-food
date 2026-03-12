pub mod api;
pub mod auth;
pub mod health;
pub mod identity;

use crate::{
    app::AppState,
    composition::contribution::ApiRouteMount,
    error::{ErrorDetails, ErrorEnvelope, FieldIssue, FieldLocation},
    http::{self, PageMeta},
    readiness::DependencyChecks,
};
use axum::{Router, extract::DefaultBodyLimit};
use utoipa::{OpenApi, openapi::OpenApi as OpenApiDocument};
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
        (path = API_PREFIX, api = api::ApiGroupDoc)
    ),
    tags(
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;

pub fn router(
    route_mounts: Vec<ApiRouteMount>,
    openapi_documents: Vec<OpenApiDocument>,
) -> Router<AppState> {
    let health_router = health::router().method_not_allowed_fallback(http::method_not_allowed);
    let api_router = api::router()
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(DefaultBodyLimit::max(http::API_BODY_LIMIT_BYTES));
    let mut router = Router::new()
        .nest("/health", health_router)
        .nest(API_PREFIX, api_router)
        .fallback(http::not_found);

    for route_mount in route_mounts {
        router = router.nest(route_mount.path, route_mount.router);
    }

    let mut openapi = ApiDoc::openapi();
    for contribution in openapi_documents {
        openapi.merge(contribution);
    }

    router.merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
}
