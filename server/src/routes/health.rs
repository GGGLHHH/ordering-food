use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    readiness::{LiveResponse, ReadyResponse},
};
use axum::{Json, Router, extract::State, http::HeaderMap, routing::get};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/live", get(live))
        .route("/ready", get(ready))
}

#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    responses(
        (status = 200, description = "Service process is running", body = LiveResponse)
    )
)]
pub async fn live() -> Json<LiveResponse> {
    Json(LiveResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "All dependencies are ready", body = ReadyResponse),
        (status = 503, description = "At least one dependency is unavailable", body = ErrorEnvelope)
    )
)]
pub async fn ready(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ReadyResponse>, AppError> {
    let checks = state
        .readiness
        .check()
        .await
        .map_err(|error| error.with_request_id(extract_request_id(&headers)))?;

    Ok(Json(ReadyResponse {
        status: "ok".to_string(),
        checks,
    }))
}

fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
