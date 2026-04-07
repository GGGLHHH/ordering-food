pub mod api;
pub mod auth;
pub mod catalog;
pub mod fulfillment;
pub mod health;
pub mod identity;
pub mod ordering;

use crate::{
    app::AppState,
    composition::contribution::ApiRouteMount,
    error::{ErrorDetails, ErrorEnvelope, FieldIssue, FieldLocation},
    http::{self, PageMeta},
    readiness::DependencyChecks,
};
use axum::{Router, extract::DefaultBodyLimit};
use std::collections::BTreeMap;
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

    let mut grouped_mounts = BTreeMap::<&'static str, Router<AppState>>::new();
    for route_mount in route_mounts {
        if let Some(existing) = grouped_mounts.remove(route_mount.path) {
            grouped_mounts.insert(route_mount.path, existing.merge(route_mount.router));
        } else {
            grouped_mounts.insert(route_mount.path, route_mount.router);
        }
    }

    for (path, nested_router) in grouped_mounts {
        router = router.nest(path, nested_router);
    }

    let mut openapi = ApiDoc::openapi();
    for contribution in openapi_documents {
        openapi.merge(contribution);
    }

    router.merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
}
