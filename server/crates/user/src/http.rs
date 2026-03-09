use std::sync::Arc;

use axum::extract::FromRef;
use axum::{Json, Router, extract::State, routing::{get, post}};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};

use ordering_food_shared::{
    error::{AppError, ErrorEnvelope},
    http::{ApiJson, ApiPath, RequestContext},
};

use crate::domain::{Role, UpdateUser, User, UserStatus};
use crate::service::UserService;

pub const LOGIN_PATH: &str = "/users/login";
pub const USER_PATH: &str = "/users/{user_id}";

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<UserService>: FromRef<S>,
{
    Router::new()
        .route(LOGIN_PATH, post(login))
        .route(USER_PATH, get(get_user).patch(update_profile))
}

#[derive(OpenApi)]
#[openapi(
    paths(login, get_user, update_profile),
    components(schemas(
        LoginRequest,
        UserResponse,
        UpdateProfileRequest,
        Role,
        UserStatus,
        ErrorEnvelope,
    )),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
pub struct UserApiDoc;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Phone number (10-15 digits).
    pub phone: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub phone: String,
    pub nickname: String,
    pub avatar_url: String,
    pub role: Role,
    pub status: UserStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            phone: u.phone,
            nickname: u.nickname,
            avatar_url: u.avatar_url,
            role: u.role,
            status: u.status,
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

impl From<UpdateProfileRequest> for UpdateUser {
    fn from(req: UpdateProfileRequest) -> Self {
        Self {
            nickname: req.nickname,
            avatar_url: req.avatar_url,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct UserPath {
    pub user_id: i64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = LOGIN_PATH,
    tag = "users",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login/register successful", body = UserResponse),
        (status = 422, description = "Invalid phone number", body = ErrorEnvelope),
    )
)]
async fn login(
    context: RequestContext,
    State(svc): State<Arc<UserService>>,
    ApiJson(body): ApiJson<LoginRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = svc
        .find_or_create_by_phone(&body.phone)
        .await
        .map_err(|e| e.with_request_id(context.request_id))?;
    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    get,
    path = USER_PATH,
    tag = "users",
    params(UserPath),
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorEnvelope),
    )
)]
async fn get_user(
    context: RequestContext,
    State(svc): State<Arc<UserService>>,
    ApiPath(path): ApiPath<UserPath>,
) -> Result<Json<UserResponse>, AppError> {
    let user = svc
        .get_by_id(path.user_id)
        .await
        .map_err(|e| e.with_request_id(context.request_id))?;
    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    patch,
    path = USER_PATH,
    tag = "users",
    params(UserPath),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorEnvelope),
        (status = 422, description = "Validation error", body = ErrorEnvelope),
    )
)]
async fn update_profile(
    context: RequestContext,
    State(svc): State<Arc<UserService>>,
    ApiPath(path): ApiPath<UserPath>,
    ApiJson(body): ApiJson<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = svc
        .update_profile(path.user_id, body.into())
        .await
        .map_err(|e| e.with_request_id(context.request_id))?;
    Ok(Json(UserResponse::from(user)))
}
