pub mod health;

use crate::{app::AppState, error::ErrorEnvelope, readiness::DependencyChecks};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/health", health::router())
        .nest("/api/v1", Router::new())
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
