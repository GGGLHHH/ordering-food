use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    readiness::{LiveResponse, ReadyResponse},
};
use axum::{Extension, Json, Router, extract::State, routing::get};
use tower_http::request_id::RequestId;

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
    request_id: Option<Extension<RequestId>>,
) -> Result<Json<ReadyResponse>, AppError> {
    let request_id = request_id_to_string(request_id);

    let checks = state
        .readiness
        .check()
        .await
        .map_err(|error| error.with_request_id(request_id.clone()))?;

    Ok(Json(ReadyResponse {
        status: "ok".to_string(),
        checks,
    }))
}

fn request_id_to_string(request_id: Option<Extension<RequestId>>) -> Option<String> {
    request_id.and_then(|Extension(request_id)| {
        request_id
            .header_value()
            .to_str()
            .ok()
            .map(ToOwned::to_owned)
    })
}
