use crate::{
    app::AppState,
    error::{AppError, ErrorDetails, ErrorEnvelope, FieldIssue, FieldLocation},
    http::{self, ApiJson, ApiPath, RequestContext},
};
use axum::{
    Extension, Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, patch, post},
};
use ordering_food_identity_application::{
    ApplicationError, BindUserIdentityInput, CreateUserIdentityInput, CreateUserInput,
    DisableUserInput, IdentityModule, SoftDeleteUserInput, UpdateUserProfileInput,
    UserIdentityReadModel, UserProfileReadModel, UserReadModel,
};
use ordering_food_identity_domain::{User, UserId};
use ordering_food_shared_kernel::Identifier;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use ts_rs::TS;
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const IDENTITY_ROUTE_PREFIX: &str = "/api/identity";
pub(crate) const IDENTITY_USERS_PATH: &str = "/api/identity/users";
pub(crate) const IDENTITY_USER_PATH: &str = "/api/identity/users/{user_id}";
pub(crate) const IDENTITY_USER_PROFILE_PATH: &str = "/api/identity/users/{user_id}/profile";
pub(crate) const IDENTITY_USER_IDENTITIES_PATH: &str = "/api/identity/users/{user_id}/identities";
pub(crate) const IDENTITY_USER_DISABLE_PATH: &str = "/api/identity/users/{user_id}/disable";
pub(crate) const IDENTITY_USER_SOFT_DELETE_PATH: &str = "/api/identity/users/{user_id}/soft-delete";
const USERS_ROUTE_PATH: &str = "/users";
const USER_ROUTE_PATH: &str = "/users/{user_id}";
const USER_PROFILE_ROUTE_PATH: &str = "/users/{user_id}/profile";
const USER_IDENTITIES_ROUTE_PATH: &str = "/users/{user_id}/identities";
const USER_DISABLE_ROUTE_PATH: &str = "/users/{user_id}/disable";
const USER_SOFT_DELETE_ROUTE_PATH: &str = "/users/{user_id}/soft-delete";

pub fn router(module: Arc<IdentityModule>) -> Router<AppState> {
    Router::new()
        .route(USERS_ROUTE_PATH, post(create_user))
        .route(USER_ROUTE_PATH, get(get_user))
        .route(USER_PROFILE_ROUTE_PATH, patch(update_user_profile))
        .route(USER_IDENTITIES_ROUTE_PATH, post(bind_user_identity))
        .route(USER_DISABLE_ROUTE_PATH, post(disable_user))
        .route(USER_SOFT_DELETE_ROUTE_PATH, post(soft_delete_user))
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(DefaultBodyLimit::max(http::API_BODY_LIMIT_BYTES))
        .layer(Extension(module))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        get_user,
        update_user_profile,
        bind_user_identity,
        disable_user,
        soft_delete_user,
    ),
    components(
        schemas(
            ErrorEnvelope,
            ErrorDetails,
            FieldIssue,
            FieldLocation,
            CreateIdentityUserRequest,
            CreateIdentityUserIdentityRequest,
            UpdateIdentityUserProfileRequest,
            BindIdentityUserIdentityRequest,
            IdentityUserPath,
            IdentityUserResponse,
            IdentityUserProfileResponse,
            IdentityUserIdentityResponse,
        )
    ),
    tags(
        (name = "identity", description = "Identity HTTP contract endpoints")
    )
)]
pub struct IdentityApiDoc;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct CreateIdentityUserRequest {
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub given_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub family_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avatar_url: Option<String>,
    pub identities: Vec<CreateIdentityUserIdentityRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct CreateIdentityUserIdentityRequest {
    pub identity_type: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct UpdateIdentityUserProfileRequest {
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub given_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub family_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct BindIdentityUserIdentityRequest {
    pub identity_type: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema, TS)]
#[into_params(parameter_in = Path)]
pub struct IdentityUserPath {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct IdentityUserResponse {
    pub user_id: String,
    pub status: String,
    pub profile: IdentityUserProfileResponse,
    pub identities: Vec<IdentityUserIdentityResponse>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct IdentityUserProfileResponse {
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub given_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub family_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct IdentityUserIdentityResponse {
    pub identity_type: String,
    pub identifier_normalized: String,
    pub bound_at: String,
}

#[utoipa::path(
    post,
    path = IDENTITY_USERS_PATH,
    tag = "identity",
    request_body = CreateIdentityUserRequest,
    responses(
        (status = 201, description = "Create a new identity user", body = IdentityUserResponse),
        (status = 400, description = "Invalid request body", body = ErrorEnvelope),
        (status = 409, description = "Identity conflicts with an existing user", body = ErrorEnvelope),
        (status = 413, description = "Request body exceeds limit", body = ErrorEnvelope),
        (status = 415, description = "Unsupported media type", body = ErrorEnvelope),
        (status = 422, description = "Body validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn create_user(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    ApiJson(payload): ApiJson<CreateIdentityUserRequest>,
) -> Result<(StatusCode, Json<IdentityUserResponse>), AppError> {
    let user = module
        .create_user
        .execute(CreateUserInput {
            display_name: payload.display_name,
            given_name: payload.given_name,
            family_name: payload.family_name,
            avatar_url: payload.avatar_url,
            identities: payload
                .identities
                .into_iter()
                .map(|identity| CreateUserIdentityInput {
                    identity_type: identity.identity_type,
                    identifier: identity.identifier,
                })
                .collect(),
            password: payload.password,
        })
        .await
        .map_err(|error| map_application_error(error, context.request_id.clone()))?;

    let response = map_domain_user_to_response(user)
        .map_err(|error| error.with_request_id(context.request_id.clone()))?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = IDENTITY_USER_PATH,
    tag = "identity",
    params(IdentityUserPath),
    responses(
        (status = 200, description = "Fetch a user by id", body = IdentityUserResponse),
        (status = 400, description = "Invalid path parameters", body = ErrorEnvelope),
        (status = 404, description = "User was not found", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_user(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<IdentityUserPath>,
) -> Result<Json<IdentityUserResponse>, AppError> {
    let user = module
        .user_queries
        .get_by_id(&UserId::new(path.user_id))
        .await
        .map_err(|error| map_application_error(error, context.request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("user was not found").with_request_id(context.request_id.clone())
        })?;

    let response = map_read_model_to_response(user)
        .map_err(|error| error.with_request_id(context.request_id.clone()))?;

    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = IDENTITY_USER_PROFILE_PATH,
    tag = "identity",
    params(IdentityUserPath),
    request_body = UpdateIdentityUserProfileRequest,
    responses(
        (status = 200, description = "Update the user profile", body = IdentityUserResponse),
        (status = 400, description = "Invalid request", body = ErrorEnvelope),
        (status = 404, description = "User was not found", body = ErrorEnvelope),
        (status = 413, description = "Request body exceeds limit", body = ErrorEnvelope),
        (status = 415, description = "Unsupported media type", body = ErrorEnvelope),
        (status = 422, description = "Body validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn update_user_profile(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<IdentityUserPath>,
    ApiJson(payload): ApiJson<UpdateIdentityUserProfileRequest>,
) -> Result<Json<IdentityUserResponse>, AppError> {
    module
        .update_user_profile
        .execute(UpdateUserProfileInput {
            user_id: path.user_id.clone(),
            display_name: payload.display_name,
            given_name: payload.given_name,
            family_name: payload.family_name,
            avatar_url: payload.avatar_url,
        })
        .await
        .map_err(|error| map_application_error(error, context.request_id.clone()))?;

    let response = load_user_response(&module, &path.user_id, context.request_id.clone()).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = IDENTITY_USER_IDENTITIES_PATH,
    tag = "identity",
    params(IdentityUserPath),
    request_body = BindIdentityUserIdentityRequest,
    responses(
        (status = 200, description = "Bind a new identity to the user", body = IdentityUserResponse),
        (status = 400, description = "Invalid request", body = ErrorEnvelope),
        (status = 404, description = "User was not found", body = ErrorEnvelope),
        (status = 409, description = "Identity conflicts with an existing user", body = ErrorEnvelope),
        (status = 413, description = "Request body exceeds limit", body = ErrorEnvelope),
        (status = 415, description = "Unsupported media type", body = ErrorEnvelope),
        (status = 422, description = "Body validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn bind_user_identity(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<IdentityUserPath>,
    ApiJson(payload): ApiJson<BindIdentityUserIdentityRequest>,
) -> Result<Json<IdentityUserResponse>, AppError> {
    module
        .bind_user_identity
        .execute(BindUserIdentityInput {
            user_id: path.user_id.clone(),
            identity_type: payload.identity_type,
            identifier: payload.identifier,
        })
        .await
        .map_err(|error| map_application_error(error, context.request_id.clone()))?;

    let response = load_user_response(&module, &path.user_id, context.request_id.clone()).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = IDENTITY_USER_DISABLE_PATH,
    tag = "identity",
    params(IdentityUserPath),
    responses(
        (status = 200, description = "Disable the user", body = IdentityUserResponse),
        (status = 400, description = "Invalid path parameters", body = ErrorEnvelope),
        (status = 404, description = "User was not found", body = ErrorEnvelope),
        (status = 409, description = "User can no longer be disabled", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn disable_user(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<IdentityUserPath>,
) -> Result<Json<IdentityUserResponse>, AppError> {
    module
        .disable_user
        .execute(DisableUserInput {
            user_id: path.user_id.clone(),
        })
        .await
        .map_err(|error| map_application_error(error, context.request_id.clone()))?;

    let response = load_user_response(&module, &path.user_id, context.request_id.clone()).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = IDENTITY_USER_SOFT_DELETE_PATH,
    tag = "identity",
    params(IdentityUserPath),
    responses(
        (status = 200, description = "Soft delete the user", body = IdentityUserResponse),
        (status = 400, description = "Invalid path parameters", body = ErrorEnvelope),
        (status = 404, description = "User was not found", body = ErrorEnvelope),
        (status = 409, description = "User has already been soft deleted", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn soft_delete_user(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    ApiPath(path): ApiPath<IdentityUserPath>,
) -> Result<Json<IdentityUserResponse>, AppError> {
    module
        .soft_delete_user
        .execute(SoftDeleteUserInput {
            user_id: path.user_id.clone(),
        })
        .await
        .map_err(|error| map_application_error(error, context.request_id.clone()))?;

    let response = load_user_response(&module, &path.user_id, context.request_id.clone()).await?;
    Ok(Json(response))
}

fn map_application_error(error: ApplicationError, request_id: Option<String>) -> AppError {
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
        ApplicationError::Unauthorized { message } => {
            AppError::unauthorized(message).with_request_id(request_id)
        }
        ApplicationError::Unexpected { .. } => {
            AppError::internal("internal server error").with_request_id(request_id)
        }
    }
}

async fn load_user_response(
    module: &IdentityModule,
    user_id: &str,
    request_id: Option<String>,
) -> Result<IdentityUserResponse, AppError> {
    let user = module
        .user_queries
        .get_by_id(&UserId::new(user_id.to_string()))
        .await
        .map_err(|error| map_application_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::internal("internal server error").with_request_id(request_id.clone())
        })?;

    map_read_model_to_response(user).map_err(|error| error.with_request_id(request_id))
}

fn map_domain_user_to_response(user: User) -> Result<IdentityUserResponse, AppError> {
    Ok(IdentityUserResponse {
        user_id: user.id().as_str().to_string(),
        status: user.status().as_str().to_string(),
        profile: IdentityUserProfileResponse {
            display_name: user.profile().display_name().to_string(),
            given_name: user.profile().given_name().map(ToOwned::to_owned),
            family_name: user.profile().family_name().map(ToOwned::to_owned),
            avatar_url: user.profile().avatar_url().map(ToOwned::to_owned),
        },
        identities: user
            .identities()
            .iter()
            .map(|identity| {
                Ok(IdentityUserIdentityResponse {
                    identity_type: identity.identity_type().as_str().to_string(),
                    identifier_normalized: identity.identifier_normalized().as_str().to_string(),
                    bound_at: format_timestamp(identity.bound_at())?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?,
        created_at: format_timestamp(user.created_at())?,
        updated_at: format_timestamp(user.updated_at())?,
        deleted_at: user.deleted_at().map(format_timestamp).transpose()?,
    })
}

fn map_read_model_to_response(user: UserReadModel) -> Result<IdentityUserResponse, AppError> {
    Ok(IdentityUserResponse {
        user_id: user.user_id,
        status: user.status,
        profile: map_profile_read_model(user.profile),
        identities: user
            .identities
            .into_iter()
            .map(map_identity_read_model)
            .collect::<Result<Vec<_>, AppError>>()?,
        created_at: format_timestamp(user.created_at)?,
        updated_at: format_timestamp(user.updated_at)?,
        deleted_at: user.deleted_at.map(format_timestamp).transpose()?,
    })
}

fn map_profile_read_model(profile: UserProfileReadModel) -> IdentityUserProfileResponse {
    IdentityUserProfileResponse {
        display_name: profile.display_name,
        given_name: profile.given_name,
        family_name: profile.family_name,
        avatar_url: profile.avatar_url,
    }
}

fn map_identity_read_model(
    identity: UserIdentityReadModel,
) -> Result<IdentityUserIdentityResponse, AppError> {
    Ok(IdentityUserIdentityResponse {
        identity_type: identity.identity_type,
        identifier_normalized: identity.identifier_normalized,
        bound_at: format_timestamp(identity.bound_at)?,
    })
}

fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, AppError> {
    timestamp
        .format(&Rfc3339)
        .map_err(|error| AppError::internal_with_source("internal server error", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppState,
        readiness::{DependencyChecks, ReadinessProbe},
    };
    use async_trait::async_trait;
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{HeaderName, Method, Request, StatusCode},
        response::Response,
    };
    use ordering_food_identity_application::{
        AccessTokenClaims, ApplicationError, Clock, CredentialRepository, IdGenerator,
        PasswordHasher, RefreshTokenStore, StoredCredential, TokenPair, TokenService,
        TransactionContext, TransactionManager, UserReadRepository, UserRepository,
    };
    use ordering_food_identity_domain::{
        IdentityType, NormalizedIdentifier, User, UserIdentity, UserProfile, UserStatus,
    };
    use ordering_food_shared_kernel::Timestamp;
    use serde_json::Value;
    use std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;
    use tower::ServiceExt;
    use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

    #[derive(Default)]
    struct FakeTransactionContext;

    impl TransactionContext for FakeTransactionContext {
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
            self
        }
    }

    #[derive(Default)]
    struct FakeTransactionManager;

    #[async_trait]
    impl TransactionManager for FakeTransactionManager {
        async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
            Ok(Box::new(FakeTransactionContext))
        }

        async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    struct FakeClock {
        now: Timestamp,
    }

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FakeIdGenerator {
        next_id: ordering_food_identity_domain::UserId,
    }

    impl IdGenerator for FakeIdGenerator {
        fn next_user_id(&self) -> ordering_food_identity_domain::UserId {
            self.next_id.clone()
        }
    }

    #[derive(Default)]
    struct FakeRepository {
        users: Mutex<HashMap<String, User>>,
    }

    impl FakeRepository {
        fn seed(&self, user: User) {
            self.users
                .lock()
                .unwrap()
                .insert(user.id().as_str().to_string(), user);
        }
    }

    #[async_trait]
    impl UserRepository for FakeRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            user_id: &ordering_food_identity_domain::UserId,
        ) -> Result<Option<User>, ApplicationError> {
            Ok(self.users.lock().unwrap().get(user_id.as_str()).cloned())
        }

        async fn find_by_identity(
            &self,
            _tx: &mut dyn TransactionContext,
            identity_type: &IdentityType,
            identifier: &NormalizedIdentifier,
        ) -> Result<Option<User>, ApplicationError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .values()
                .find(|user| {
                    user.identities().iter().any(|identity| {
                        identity.identity_type() == identity_type
                            && identity.identifier_normalized() == identifier
                    })
                })
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            user: &User,
        ) -> Result<(), ApplicationError> {
            self.users
                .lock()
                .unwrap()
                .insert(user.id().as_str().to_string(), user.clone());
            Ok(())
        }

        async fn update(
            &self,
            _tx: &mut dyn TransactionContext,
            user: &User,
        ) -> Result<(), ApplicationError> {
            self.users
                .lock()
                .unwrap()
                .insert(user.id().as_str().to_string(), user.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl UserReadRepository for FakeRepository {
        async fn get_by_id(
            &self,
            user_id: &ordering_food_identity_domain::UserId,
        ) -> Result<Option<UserReadModel>, ApplicationError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .get(user_id.as_str())
                .cloned()
                .map(|user| UserReadModel {
                    user_id: user.id().as_str().to_string(),
                    status: user.status().as_str().to_string(),
                    profile: UserProfileReadModel {
                        display_name: user.profile().display_name().to_string(),
                        given_name: user.profile().given_name().map(ToOwned::to_owned),
                        family_name: user.profile().family_name().map(ToOwned::to_owned),
                        avatar_url: user.profile().avatar_url().map(ToOwned::to_owned),
                    },
                    identities: user
                        .identities()
                        .iter()
                        .map(|identity| UserIdentityReadModel {
                            identity_type: identity.identity_type().as_str().to_string(),
                            identifier_normalized: identity
                                .identifier_normalized()
                                .as_str()
                                .to_string(),
                            bound_at: identity.bound_at(),
                        })
                        .collect(),
                    created_at: user.created_at(),
                    updated_at: user.updated_at(),
                    deleted_at: user.deleted_at(),
                }))
        }
    }

    struct StubReadiness;

    #[async_trait]
    impl ReadinessProbe for StubReadiness {
        async fn check(&self) -> Result<DependencyChecks, AppError> {
            Ok(DependencyChecks::ok("ok", "ok"))
        }
    }

    #[derive(Default)]
    struct FakePasswordHasher;

    #[async_trait]
    impl PasswordHasher for FakePasswordHasher {
        async fn hash(&self, raw: &str) -> Result<String, ApplicationError> {
            Ok(format!("hashed:{raw}"))
        }
        async fn verify(&self, _raw: &str, _hash: &str) -> Result<bool, ApplicationError> {
            Ok(true)
        }
    }

    #[derive(Default)]
    struct FakeCredentialRepository;

    #[async_trait]
    impl CredentialRepository for FakeCredentialRepository {
        async fn find_by_user_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _user_id: &ordering_food_identity_domain::UserId,
        ) -> Result<Option<StoredCredential>, ApplicationError> {
            Ok(None)
        }
        async fn upsert(
            &self,
            _tx: &mut dyn TransactionContext,
            _user_id: &ordering_food_identity_domain::UserId,
            _hash: &str,
            _now: ordering_food_shared_kernel::Timestamp,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    struct FakeTokenService;

    #[async_trait]
    impl TokenService for FakeTokenService {
        fn generate_token_pair(&self, user_id: &str) -> Result<TokenPair, ApplicationError> {
            Ok(TokenPair {
                access_token: format!("at-{user_id}"),
                access_token_expires_in: 900,
                refresh_token: format!("rt-{user_id}"),
                refresh_token_expires_in: 604800,
            })
        }
        fn verify_access_token(&self, _token: &str) -> Result<AccessTokenClaims, ApplicationError> {
            Err(ApplicationError::unauthorized("not implemented"))
        }
    }

    #[derive(Default)]
    struct FakeRefreshTokenStore;

    #[async_trait]
    impl RefreshTokenStore for FakeRefreshTokenStore {
        async fn store(
            &self,
            _token: &str,
            _user_id: &str,
            _ttl: u64,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
        async fn lookup(&self, _token: &str) -> Result<Option<String>, ApplicationError> {
            Ok(None)
        }
        async fn revoke(&self, _token: &str) -> Result<(), ApplicationError> {
            Ok(())
        }
        async fn revoke_all_for_user(&self, _user_id: &str) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    fn build_test_app(repository: Arc<FakeRepository>) -> Router {
        let module = Arc::new(IdentityModule::new(
            repository.clone(),
            repository,
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 08:00 UTC),
            }),
            Arc::new(FakeIdGenerator {
                next_id: ordering_food_identity_domain::UserId::new("user-123"),
            }),
            Arc::new(FakeCredentialRepository),
            Arc::new(FakePasswordHasher),
            Arc::new(FakeTokenService),
            Arc::new(FakeRefreshTokenStore),
        ));
        let request_id_header = HeaderName::from_static("x-request-id");

        Router::new()
            .nest(IDENTITY_ROUTE_PREFIX, router(module))
            .fallback(http::not_found)
            .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
            .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
            .with_state(AppState::new(Arc::new(StubReadiness)))
    }

    async fn response_json(response: Response<Body>) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn create_user_returns_created_identity_payload() {
        let app = build_test_app(Arc::new(FakeRepository::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(IDENTITY_USERS_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"display_name":"Alice","given_name":"Alice","identities":[{"identity_type":"email","identifier":"Alice@Example.com"}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-123");
        assert_eq!(body["status"], "active");
        assert_eq!(body["profile"]["display_name"], "Alice");
        assert_eq!(body["identities"][0]["identity_type"], "email");
        assert_eq!(
            body["identities"][0]["identifier_normalized"],
            "alice@example.com"
        );
        assert_eq!(body["created_at"], "2026-03-10T08:00:00Z");
    }

    #[tokio::test]
    async fn get_user_returns_read_model_payload() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(
            User::rehydrate(
                ordering_food_identity_domain::UserId::new("user-7"),
                UserStatus::Disabled,
                UserProfile::new("Bob", Some("Bob".to_string()), None, None).unwrap(),
                vec![UserIdentity::new(
                    IdentityType::Email,
                    NormalizedIdentifier::new("bob@example.com").unwrap(),
                    datetime!(2026-03-10 07:00 UTC),
                )],
                datetime!(2026-03-10 06:00 UTC),
                datetime!(2026-03-10 09:00 UTC),
                None,
            )
            .unwrap(),
        );
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/identity/users/user-7")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-7");
        assert_eq!(body["status"], "disabled");
        assert_eq!(body["profile"]["display_name"], "Bob");
        assert_eq!(
            body["identities"][0]["identifier_normalized"],
            "bob@example.com"
        );
        assert_eq!(body["updated_at"], "2026-03-10T09:00:00Z");
    }

    #[tokio::test]
    async fn update_user_profile_returns_updated_payload() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            ordering_food_identity_domain::UserId::new("user-profile"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 06:00 UTC),
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PATCH)
                    .uri("/api/identity/users/user-profile/profile")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"display_name":"Alice Chen","family_name":"Chen"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-profile");
        assert_eq!(body["profile"]["display_name"], "Alice Chen");
        assert_eq!(body["profile"]["family_name"], "Chen");
        assert_eq!(body["updated_at"], "2026-03-10T08:00:00Z");
    }

    #[tokio::test]
    async fn bind_user_identity_returns_updated_payload() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            ordering_food_identity_domain::UserId::new("user-bind"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 06:00 UTC),
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/identity/users/user-bind/identities")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"identity_type":"phone","identifier":"13800000000"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-bind");
        assert_eq!(body["identities"][0]["identity_type"], "phone");
        assert_eq!(
            body["identities"][0]["identifier_normalized"],
            "13800000000"
        );
        assert_eq!(body["updated_at"], "2026-03-10T08:00:00Z");
    }

    #[tokio::test]
    async fn disable_user_returns_disabled_payload() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            ordering_food_identity_domain::UserId::new("user-disable"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 06:00 UTC),
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/identity/users/user-disable/disable")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-disable");
        assert_eq!(body["status"], "disabled");
        assert_eq!(body["updated_at"], "2026-03-10T08:00:00Z");
    }

    #[tokio::test]
    async fn disable_user_returns_conflict_envelope_when_user_is_already_disabled() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(
            User::rehydrate(
                ordering_food_identity_domain::UserId::new("user-disable"),
                UserStatus::Disabled,
                UserProfile::new("Alice", None, None, None).unwrap(),
                Vec::new(),
                datetime!(2026-03-10 06:00 UTC),
                datetime!(2026-03-10 07:00 UTC),
                None,
            )
            .unwrap(),
        );
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/identity/users/user-disable/disable")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = response_json(response).await;
        assert_eq!(body["code"], "conflict");
        assert_eq!(body["message"], "user can no longer be disabled");
    }

    #[tokio::test]
    async fn soft_delete_user_returns_deleted_payload() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            ordering_food_identity_domain::UserId::new("user-soft-delete"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 06:00 UTC),
        ));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/identity/users/user-soft-delete/soft-delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-soft-delete");
        assert_eq!(body["status"], "disabled");
        assert_eq!(body["updated_at"], "2026-03-10T08:00:00Z");
        assert_eq!(body["deleted_at"], "2026-03-10T08:00:00Z");
    }

    #[tokio::test]
    async fn soft_delete_user_conflict_returns_conflict_envelope() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(
            User::rehydrate(
                ordering_food_identity_domain::UserId::new("user-soft-deleted"),
                UserStatus::Disabled,
                UserProfile::new("Alice", None, None, None).unwrap(),
                Vec::new(),
                datetime!(2026-03-10 06:00 UTC),
                datetime!(2026-03-10 07:00 UTC),
                Some(datetime!(2026-03-10 07:00 UTC)),
            )
            .unwrap(),
        );
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/identity/users/user-soft-deleted/soft-delete")
                    .header("x-request-id", "req-soft-delete-conflict")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = response_json(response).await;
        assert_eq!(body["code"], "conflict");
        assert_eq!(body["message"], "user is already soft deleted");
        assert_eq!(body["request_id"], "req-soft-delete-conflict");
    }

    #[tokio::test]
    async fn create_user_conflict_returns_conflict_envelope() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(
            User::rehydrate(
                ordering_food_identity_domain::UserId::new("user-existing"),
                UserStatus::Active,
                UserProfile::new("Existing", None, None, None).unwrap(),
                vec![UserIdentity::new(
                    IdentityType::Email,
                    NormalizedIdentifier::new("alice@example.com").unwrap(),
                    datetime!(2026-03-10 06:00 UTC),
                )],
                datetime!(2026-03-10 06:00 UTC),
                datetime!(2026-03-10 06:00 UTC),
                None,
            )
            .unwrap(),
        );
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(IDENTITY_USERS_PATH)
                    .header("content-type", "application/json")
                    .header("x-request-id", "req-identity-conflict")
                    .body(Body::from(
                        r#"{"display_name":"Alice","identities":[{"identity_type":"email","identifier":"Alice@Example.com"}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = response_json(response).await;
        assert_eq!(body["code"], "conflict");
        assert_eq!(body["request_id"], "req-identity-conflict");
    }

    #[tokio::test]
    async fn get_user_not_found_returns_not_found_envelope() {
        let app = build_test_app(Arc::new(FakeRepository::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/identity/users/missing")
                    .header("x-request-id", "req-identity-missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response_json(response).await;
        assert_eq!(body["code"], "not_found");
        assert_eq!(body["message"], "user was not found");
        assert_eq!(body["request_id"], "req-identity-missing");
    }
}
