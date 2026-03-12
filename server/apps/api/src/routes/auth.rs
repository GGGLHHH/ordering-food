use crate::{
    app::AppState,
    config::AuthSettings,
    error::{AppError, ErrorEnvelope},
    http::{self, ApiJson, AuthenticatedUser, RequestContext},
};
use axum::{
    Extension, Json, Router,
    extract::DefaultBodyLimit,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    routing::{get, post},
};
use ordering_food_identity_application::{
    ApplicationError, IdentityModule, LoginInput, LogoutInput, RefreshTokenInput,
};
use ordering_food_identity_domain::UserId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use utoipa::{OpenApi, ToSchema};

pub(crate) const AUTH_ROUTE_PREFIX: &str = "/api/auth";
pub(crate) const AUTH_LOGIN_PATH: &str = "/api/auth/login";
pub(crate) const AUTH_REFRESH_PATH: &str = "/api/auth/refresh";
pub(crate) const AUTH_LOGOUT_PATH: &str = "/api/auth/logout";
pub(crate) const AUTH_ME_PATH: &str = "/api/auth/me";

const LOGIN_ROUTE_PATH: &str = "/login";
const REFRESH_ROUTE_PATH: &str = "/refresh";
const LOGOUT_ROUTE_PATH: &str = "/logout";
const ME_ROUTE_PATH: &str = "/me";

pub fn router(module: Arc<IdentityModule>, auth_settings: AuthSettings) -> Router<AppState> {
    Router::new()
        .route(LOGIN_ROUTE_PATH, post(login))
        .route(REFRESH_ROUTE_PATH, post(refresh))
        .route(LOGOUT_ROUTE_PATH, post(logout))
        .route(ME_ROUTE_PATH, get(me))
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(DefaultBodyLimit::max(http::API_BODY_LIMIT_BYTES))
        .layer(Extension(module))
        .layer(Extension(Arc::new(auth_settings)))
}

#[derive(OpenApi)]
#[openapi(
    paths(login, refresh, logout, me),
    components(schemas(
        ErrorEnvelope,
        LoginRequest,
        AuthResponse,
        AuthMeResponse,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct AuthApiDoc;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct LoginRequest {
    pub identity_type: String,
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct AuthResponse {
    pub user_id: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct AuthMeResponse {
    pub user_id: String,
    pub status: String,
    pub display_name: String,
}

#[utoipa::path(
    post,
    path = AUTH_LOGIN_PATH,
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorEnvelope),
        (status = 400, description = "Invalid request body", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn login(
    Extension(module): Extension<Arc<IdentityModule>>,
    Extension(auth_settings): Extension<Arc<AuthSettings>>,
    context: RequestContext,
    ApiJson(payload): ApiJson<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    let output = module
        .login
        .execute(LoginInput {
            identity_type: payload.identity_type,
            identifier: payload.identifier,
            password: payload.password,
        })
        .await
        .map_err(|error| map_auth_error(error, context.request_id.clone()))?;

    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        build_access_cookie(&output.token_pair.access_token, &auth_settings),
    );
    headers.append(
        SET_COOKIE,
        build_refresh_cookie(&output.token_pair.refresh_token, &auth_settings),
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(AuthResponse {
            user_id: output.user_id,
            expires_in: output.token_pair.access_token_expires_in,
        }),
    ))
}

#[utoipa::path(
    post,
    path = AUTH_REFRESH_PATH,
    tag = "auth",
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Invalid refresh token", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn refresh(
    Extension(module): Extension<Arc<IdentityModule>>,
    Extension(auth_settings): Extension<Arc<AuthSettings>>,
    context: RequestContext,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    let refresh_token = extract_cookie(&headers, "refresh_token").ok_or_else(|| {
        AppError::unauthorized("missing refresh token").with_request_id(context.request_id.clone())
    })?;

    let output = module
        .refresh_token
        .execute(RefreshTokenInput { refresh_token })
        .await
        .map_err(|error| map_auth_error(error, context.request_id.clone()))?;

    let mut response_headers = HeaderMap::new();
    response_headers.append(
        SET_COOKIE,
        build_access_cookie(&output.token_pair.access_token, &auth_settings),
    );
    response_headers.append(
        SET_COOKIE,
        build_refresh_cookie(&output.token_pair.refresh_token, &auth_settings),
    );

    Ok((
        StatusCode::OK,
        response_headers,
        Json(AuthResponse {
            user_id: output.user_id,
            expires_in: output.token_pair.access_token_expires_in,
        }),
    ))
}

#[utoipa::path(
    post,
    path = AUTH_LOGOUT_PATH,
    tag = "auth",
    responses(
        (status = 204, description = "Logged out"),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn logout(
    Extension(module): Extension<Arc<IdentityModule>>,
    Extension(auth_settings): Extension<Arc<AuthSettings>>,
    context: RequestContext,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap), AppError> {
    if let Some(refresh_token) = extract_cookie(&headers, "refresh_token") {
        module
            .logout
            .execute(LogoutInput { refresh_token })
            .await
            .map_err(|error| map_auth_error(error, context.request_id.clone()))?;
    }

    let mut response_headers = HeaderMap::new();
    response_headers.append(
        SET_COOKIE,
        clear_cookie("access_token", "/api", &auth_settings),
    );
    response_headers.append(
        SET_COOKIE,
        clear_cookie("refresh_token", "/api/auth", &auth_settings),
    );

    Ok((StatusCode::NO_CONTENT, response_headers))
}

#[utoipa::path(
    get,
    path = AUTH_ME_PATH,
    tag = "auth",
    responses(
        (status = 200, description = "Current user info", body = AuthMeResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn me(
    Extension(module): Extension<Arc<IdentityModule>>,
    context: RequestContext,
    user: AuthenticatedUser,
) -> Result<Json<AuthMeResponse>, AppError> {
    let read_model = module
        .user_queries
        .get_by_id(&UserId::new(&user.user_id))
        .await
        .map_err(|error| map_auth_error(error, context.request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("user not found").with_request_id(context.request_id.clone())
        })?;

    Ok(Json(AuthMeResponse {
        user_id: read_model.user_id,
        status: read_model.status,
        display_name: read_model.profile.display_name,
    }))
}

fn map_auth_error(error: ApplicationError, request_id: Option<String>) -> AppError {
    match error {
        ApplicationError::Validation { message } => {
            AppError::validation_error(message).with_request_id(request_id)
        }
        ApplicationError::Unauthorized { message } => {
            AppError::unauthorized(message).with_request_id(request_id)
        }
        ApplicationError::NotFound { message } => {
            AppError::not_found(message).with_request_id(request_id)
        }
        ApplicationError::Conflict { message } => {
            AppError::conflict(message).with_request_id(request_id)
        }
        ApplicationError::Unexpected { .. } => {
            AppError::internal("internal server error").with_request_id(request_id)
        }
    }
}

fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|header| {
            header.split(';').find_map(|pair| {
                let pair = pair.trim();
                let (key, value) = pair.split_once('=')?;
                if key.trim() == name {
                    Some(value.trim().to_string())
                } else {
                    None
                }
            })
        })
}

fn build_access_cookie(token: &str, settings: &AuthSettings) -> HeaderValue {
    let secure = if settings.cookie_secure {
        "; Secure"
    } else {
        ""
    };
    let domain = if settings.cookie_domain.is_empty() {
        String::new()
    } else {
        format!("; Domain={}", settings.cookie_domain)
    };
    let value = format!(
        "access_token={token}; HttpOnly; SameSite=Strict; Path=/api; Max-Age={}{secure}{domain}",
        settings.access_token_ttl_seconds
    );
    HeaderValue::from_str(&value).expect("valid cookie header")
}

fn build_refresh_cookie(token: &str, settings: &AuthSettings) -> HeaderValue {
    let secure = if settings.cookie_secure {
        "; Secure"
    } else {
        ""
    };
    let domain = if settings.cookie_domain.is_empty() {
        String::new()
    } else {
        format!("; Domain={}", settings.cookie_domain)
    };
    let value = format!(
        "refresh_token={token}; HttpOnly; SameSite=Strict; Path=/api/auth; Max-Age={}{secure}{domain}",
        settings.refresh_token_ttl_seconds
    );
    HeaderValue::from_str(&value).expect("valid cookie header")
}

fn clear_cookie(name: &str, path: &str, settings: &AuthSettings) -> HeaderValue {
    let secure = if settings.cookie_secure {
        "; Secure"
    } else {
        ""
    };
    let domain = if settings.cookie_domain.is_empty() {
        String::new()
    } else {
        format!("; Domain={}", settings.cookie_domain)
    };
    let value =
        format!("{name}=; HttpOnly; SameSite=Strict; Path={path}; Max-Age=0{secure}{domain}");
    HeaderValue::from_str(&value).expect("valid cookie header")
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
        Extension, Router,
        body::{Body, to_bytes},
        http::{HeaderName, Request, StatusCode, header::SET_COOKIE},
        response::Response,
    };
    use ordering_food_identity_application::{
        AccessTokenClaims, ApplicationError, Clock, CredentialRepository, DisableUserInput,
        IdGenerator, PasswordHasher, RefreshTokenStore, StoredCredential, TokenPair, TokenService,
        TransactionContext, TransactionManager, UserIdentityReadModel, UserProfileReadModel,
        UserReadModel, UserReadRepository, UserRepository,
    };
    use ordering_food_identity_domain::{
        IdentityType, NormalizedIdentifier, User, UserId, UserIdentity, UserProfile, UserStatus,
    };
    use ordering_food_shared_kernel::{Identifier, Timestamp};
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

    struct FakeIdGenerator;

    impl IdGenerator for FakeIdGenerator {
        fn next_user_id(&self) -> UserId {
            UserId::new("generated-user")
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
            user_id: &UserId,
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
            user_id: &UserId,
        ) -> Result<Option<UserReadModel>, ApplicationError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(user_id.as_str()).map(|user| UserReadModel {
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

    #[derive(Default)]
    struct FakeCredentialRepository {
        credentials: Mutex<HashMap<String, StoredCredential>>,
    }

    impl FakeCredentialRepository {
        fn seed(&self, credential: StoredCredential) {
            self.credentials
                .lock()
                .unwrap()
                .insert(credential.user_id.clone(), credential);
        }
    }

    #[async_trait]
    impl CredentialRepository for FakeCredentialRepository {
        async fn find_by_user_id(
            &self,
            _tx: &mut dyn TransactionContext,
            user_id: &UserId,
        ) -> Result<Option<StoredCredential>, ApplicationError> {
            Ok(self
                .credentials
                .lock()
                .unwrap()
                .get(user_id.as_str())
                .cloned())
        }

        async fn upsert(
            &self,
            _tx: &mut dyn TransactionContext,
            user_id: &UserId,
            password_hash: &str,
            now: Timestamp,
        ) -> Result<(), ApplicationError> {
            self.credentials.lock().unwrap().insert(
                user_id.as_str().to_string(),
                StoredCredential {
                    user_id: user_id.as_str().to_string(),
                    password_hash: password_hash.to_string(),
                    created_at: now,
                    updated_at: now,
                },
            );
            Ok(())
        }
    }

    struct FakePasswordHasher;

    #[async_trait]
    impl PasswordHasher for FakePasswordHasher {
        async fn hash(&self, raw_password: &str) -> Result<String, ApplicationError> {
            Ok(format!("hashed:{raw_password}"))
        }

        async fn verify(&self, raw_password: &str, hash: &str) -> Result<bool, ApplicationError> {
            Ok(hash == format!("hashed:{raw_password}"))
        }
    }

    #[derive(Default)]
    struct FakeTokenService {
        issued_tokens: Mutex<HashMap<String, String>>,
        counter: Mutex<u32>,
    }

    #[async_trait]
    impl TokenService for FakeTokenService {
        fn generate_token_pair(&self, user_id: &str) -> Result<TokenPair, ApplicationError> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            let access_token = format!("access-{user_id}-{}", *counter);
            self.issued_tokens
                .lock()
                .unwrap()
                .insert(access_token.clone(), user_id.to_string());

            Ok(TokenPair {
                access_token,
                access_token_expires_in: 900,
                refresh_token: format!("refresh-{user_id}-{}", *counter),
                refresh_token_expires_in: 604800,
            })
        }

        fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, ApplicationError> {
            let user_id = self
                .issued_tokens
                .lock()
                .unwrap()
                .get(token)
                .cloned()
                .ok_or_else(|| ApplicationError::unauthorized("invalid or expired access token"))?;

            Ok(AccessTokenClaims { user_id, exp: 900 })
        }
    }

    #[derive(Default)]
    struct FakeRefreshTokenStore {
        tokens: Mutex<HashMap<String, String>>,
    }

    #[async_trait]
    impl RefreshTokenStore for FakeRefreshTokenStore {
        async fn store(
            &self,
            token: &str,
            user_id: &str,
            _ttl_seconds: u64,
        ) -> Result<(), ApplicationError> {
            self.tokens
                .lock()
                .unwrap()
                .insert(token.to_string(), user_id.to_string());
            Ok(())
        }

        async fn lookup(&self, token: &str) -> Result<Option<String>, ApplicationError> {
            Ok(self.tokens.lock().unwrap().get(token).cloned())
        }

        async fn revoke(&self, token: &str) -> Result<(), ApplicationError> {
            self.tokens.lock().unwrap().remove(token);
            Ok(())
        }

        async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), ApplicationError> {
            self.tokens
                .lock()
                .unwrap()
                .retain(|_, value| value != user_id);
            Ok(())
        }
    }

    struct StubReadiness;

    #[async_trait]
    impl ReadinessProbe for StubReadiness {
        async fn check(&self) -> Result<DependencyChecks, AppError> {
            Ok(DependencyChecks::ok("ok", "ok"))
        }
    }

    fn auth_settings() -> AuthSettings {
        AuthSettings {
            jwt_secret: "test-secret".to_string(),
            access_token_ttl_seconds: 900,
            refresh_token_ttl_seconds: 604800,
            cookie_domain: String::new(),
            cookie_secure: false,
        }
    }

    fn make_user(user_id: &str, email: &str, status: UserStatus) -> User {
        let created_at = datetime!(2026-03-10 06:00 UTC);
        let mut user = User::create(
            UserId::new(user_id),
            UserProfile::new("Alice", None, None, None).unwrap(),
            created_at,
        );
        user.bind_identity(
            UserIdentity::new(
                IdentityType::Email,
                NormalizedIdentifier::new(email).unwrap(),
                created_at,
            ),
            created_at,
        )
        .unwrap();

        if status == UserStatus::Disabled {
            user.disable(datetime!(2026-03-10 07:00 UTC)).unwrap();
        }

        user
    }

    fn build_test_app(
        repository: Arc<FakeRepository>,
        credential_repository: Arc<FakeCredentialRepository>,
        token_service: Arc<FakeTokenService>,
        refresh_token_store: Arc<FakeRefreshTokenStore>,
    ) -> (Router, Arc<IdentityModule>) {
        let module = Arc::new(IdentityModule::new(
            repository.clone(),
            repository,
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 08:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
            credential_repository,
            Arc::new(FakePasswordHasher),
            token_service.clone(),
            refresh_token_store,
        ));
        let request_id_header = HeaderName::from_static("x-request-id");
        let token_service_extension: Arc<dyn TokenService> = token_service;

        let app = Router::new()
            .nest(
                AUTH_ROUTE_PREFIX,
                router(module.clone(), auth_settings()).layer(Extension(token_service_extension)),
            )
            .fallback(http::not_found)
            .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
            .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
            .with_state(AppState::new(Arc::new(StubReadiness)));

        (app, module)
    }

    async fn response_json(response: Response<Body>) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn cookie_header(response: &Response<Body>) -> String {
        response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter_map(|value| value.split(';').next())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
            .join("; ")
    }

    #[tokio::test]
    async fn login_sets_auth_cookies_and_returns_body() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user("user-1", "alice@example.com", UserStatus::Active));
        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 06:00 UTC),
            updated_at: datetime!(2026-03-10 06:00 UTC),
        });
        let (app, _) = build_test_app(
            repository,
            credential_repository,
            Arc::new(FakeTokenService::default()),
            Arc::new(FakeRefreshTokenStore::default()),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_LOGIN_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"identity_type":"email","identifier":"alice@example.com","password":"secret123"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get_all(SET_COOKIE).iter().count(), 2);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-1");
        assert_eq!(body["expires_in"], 900);
    }

    #[tokio::test]
    async fn me_returns_current_user_for_valid_login_cookie() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user("user-1", "alice@example.com", UserStatus::Active));
        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 06:00 UTC),
            updated_at: datetime!(2026-03-10 06:00 UTC),
        });
        let (app, _) = build_test_app(
            repository,
            credential_repository,
            Arc::new(FakeTokenService::default()),
            Arc::new(FakeRefreshTokenStore::default()),
        );

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_LOGIN_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"identity_type":"email","identifier":"alice@example.com","password":"secret123"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookie = cookie_header(&login_response);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(AUTH_ME_PATH)
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["user_id"], "user-1");
        assert_eq!(body["status"], "active");
        assert_eq!(body["display_name"], "Alice");
    }

    #[tokio::test]
    async fn logout_revokes_refresh_token_and_clears_cookies() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user("user-1", "alice@example.com", UserStatus::Active));
        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 06:00 UTC),
            updated_at: datetime!(2026-03-10 06:00 UTC),
        });
        let refresh_token_store = Arc::new(FakeRefreshTokenStore::default());
        let (app, _) = build_test_app(
            repository,
            credential_repository,
            Arc::new(FakeTokenService::default()),
            refresh_token_store,
        );

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_LOGIN_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"identity_type":"email","identifier":"alice@example.com","password":"secret123"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookie = cookie_header(&login_response);

        let logout_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_LOGOUT_PATH)
                    .header("cookie", cookie.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);
        let cleared_headers = logout_response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .collect::<Vec<_>>();
        assert!(
            cleared_headers
                .iter()
                .any(|value| value.contains("access_token="))
        );
        assert!(
            cleared_headers
                .iter()
                .any(|value| value.contains("refresh_token="))
        );

        let refresh_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_REFRESH_PATH)
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
        let body = response_json(refresh_response).await;
        assert_eq!(body["code"], "unauthorized");
        assert_eq!(body["message"], "invalid refresh token");
    }

    #[tokio::test]
    async fn refresh_rejects_user_after_disable() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(make_user("user-1", "alice@example.com", UserStatus::Active));
        let credential_repository = Arc::new(FakeCredentialRepository::default());
        credential_repository.seed(StoredCredential {
            user_id: "user-1".to_string(),
            password_hash: "hashed:secret123".to_string(),
            created_at: datetime!(2026-03-10 06:00 UTC),
            updated_at: datetime!(2026-03-10 06:00 UTC),
        });
        let (app, module) = build_test_app(
            repository,
            credential_repository,
            Arc::new(FakeTokenService::default()),
            Arc::new(FakeRefreshTokenStore::default()),
        );

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_LOGIN_PATH)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"identity_type":"email","identifier":"alice@example.com","password":"secret123"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookie = cookie_header(&login_response);

        module
            .disable_user
            .execute(DisableUserInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(AUTH_REFRESH_PATH)
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = response_json(response).await;
        assert_eq!(body["code"], "unauthorized");
    }
}
