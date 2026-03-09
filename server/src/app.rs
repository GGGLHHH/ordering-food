use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, ensure};
use axum::{
    Router,
    extract::{FromRef, MatchedPath},
    http::{HeaderName, HeaderValue, Method, Request, Response, header},
};
use redis::aio::MultiplexedConnection;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, RequestId},
    trace::TraceLayer,
};
use tracing::{Span, field, info, info_span, warn};

use ordering_food_shared::config::Settings;
use ordering_food_user::{pg_repository::PgUserRepository, service::UserService};

use crate::{
    readiness::{ReadinessProbe, RuntimeReadiness},
    routes,
};

#[derive(Clone)]
pub struct AppState {
    pub readiness: Arc<dyn ReadinessProbe>,
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(readiness: Arc<dyn ReadinessProbe>, user_service: Arc<UserService>) -> Self {
        Self {
            readiness,
            user_service,
        }
    }
}

impl FromRef<AppState> for Arc<UserService> {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}

pub async fn run() -> Result<()> {
    let settings = Settings::from_env()?;

    let pg_pool = connect_postgres(&settings).await?;

    if settings.app.auto_migrate {
        sqlx::migrate!("./migrations")
            .run(&pg_pool)
            .await
            .context("failed to run database migrations")?;
    }

    let redis_client = connect_redis(&settings).await?;

    let user_repo = Arc::new(PgUserRepository::new(pg_pool.clone()));
    let user_service = Arc::new(UserService::new(user_repo));

    let readiness = Arc::new(RuntimeReadiness::new(pg_pool, redis_client));
    let app = build_router(AppState::new(readiness, user_service), &settings)?;
    let listener = tokio::net::TcpListener::bind(settings.app.bind_address())
        .await
        .with_context(|| format!("failed to bind {}", settings.app.bind_address()))?;

    info!(address = %settings.app.bind_address(), "server listening");

    axum::serve(listener, app)
        .await
        .context("server stopped unexpectedly")
}

pub fn build_router(state: AppState, settings: &Settings) -> Result<Router> {
    let request_id_header = HeaderName::from_static("x-request-id");
    let cors_layer = build_cors_layer(settings)?;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let matched_path = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str)
                .unwrap_or_else(|| request.uri().path());
            let request_id = request
                .extensions()
                .get::<RequestId>()
                .and_then(|request_id| request_id.header_value().to_str().ok())
                .unwrap_or("-");

            info_span!(
                "http.request",
                request_id,
                method = %request.method(),
                matched_path,
                status_code = field::Empty,
                latency_ms = field::Empty,
            )
        })
        .on_request(|request: &Request<_>, _span: &Span| {
            tracing::debug!(method = %request.method(), path = request.uri().path(), "request started");
        })
        .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
            let status_code = response.status().as_u16();
            let latency_ms = latency.as_millis() as u64;

            span.record("status_code", field::display(status_code));
            span.record("latency_ms", latency_ms);

            if response.status().is_server_error() {
                warn!(parent: span, "request completed with server error");
            } else if response.status().is_client_error() {
                warn!(parent: span, "request completed with client error");
            } else {
                info!(parent: span, "request completed");
            }
        })
        .on_failure(());

    Ok(routes::router(state)
        .layer(cors_layer)
        .layer(tower_http::request_id::PropagateRequestIdLayer::new(
            request_id_header.clone(),
        ))
        .layer(trace_layer)
        .layer(tower_http::request_id::SetRequestIdLayer::new(
            request_id_header,
            MakeRequestUuid,
        )))
}

async fn connect_postgres(settings: &Settings) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .connect(&settings.database.url)
        .await
        .context("failed to connect to postgres")
}

async fn connect_redis(settings: &Settings) -> Result<redis::Client> {
    let client = redis::Client::open(settings.redis.url.as_str())
        .context("failed to create redis client")?;

    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .context("failed to establish redis connection")?;

    let pong = ping_redis(&mut connection).await?;
    ensure!(pong == "PONG", "unexpected redis ping response: {pong}");

    Ok(client)
}

async fn ping_redis(connection: &mut MultiplexedConnection) -> Result<String> {
    redis::cmd("PING")
        .query_async(connection)
        .await
        .map_err(|error| anyhow!("failed to ping redis: {error}"))
}

fn build_cors_layer(settings: &Settings) -> Result<CorsLayer> {
    let request_id_header = HeaderName::from_static("x-request-id");

    let mut cors_layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            request_id_header.clone(),
        ]);

    if !settings.app.allowed_origins.is_empty() {
        let origins = settings
            .app
            .allowed_origins
            .iter()
            .map(|origin| {
                HeaderValue::from_str(origin)
                    .with_context(|| format!("invalid CORS origin configured: {origin}"))
            })
            .collect::<Result<Vec<_>>>()?;

        cors_layer = cors_layer.allow_origin(AllowOrigin::list(origins));
    }

    Ok(cors_layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        readiness::DependencyChecks,
        routes::{
            API_PREFIX,
            api::{EXAMPLE_ECHO_PATH, EXAMPLE_ITEM_PATH, EXAMPLE_SEARCH_PATH},
        },
    };
    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode},
    };
    use ordering_food_shared::{config::Settings, error::AppError};
    use ordering_food_user::{
        domain::{NewUser, Phone, UpdateUser, User},
        repository::UserRepository,
        service::UserService,
    };
    use serde_json::Value;
    use std::sync::Mutex;
    use tower::ServiceExt;
    use uuid::Uuid;

    struct MockReadiness {
        result: Mutex<Option<Result<DependencyChecks, AppError>>>,
    }

    #[async_trait]
    impl ReadinessProbe for MockReadiness {
        async fn check(&self) -> Result<DependencyChecks, AppError> {
            self.result
                .lock()
                .unwrap()
                .take()
                .expect("mock readiness result already consumed")
        }
    }

    struct MockUserRepository;

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, AppError> {
            unimplemented!()
        }
        async fn find_by_phone(&self, _phone: &Phone) -> Result<Option<User>, AppError> {
            unimplemented!()
        }
        async fn create(&self, _new_user: &NewUser) -> Result<User, AppError> {
            unimplemented!()
        }
        async fn update(&self, _id: Uuid, _update: &UpdateUser) -> Result<Option<User>, AppError> {
            unimplemented!()
        }
    }

    fn build_test_app(result: Result<DependencyChecks, AppError>) -> Router {
        let settings = Settings::from_overrides(std::iter::empty::<(String, String)>()).unwrap();
        let readiness = Arc::new(MockReadiness {
            result: Mutex::new(Some(result)),
        });
        let user_repo: Arc<dyn UserRepository> = Arc::new(MockUserRepository);
        let user_service = Arc::new(UserService::new(user_repo));

        build_router(AppState::new(readiness, user_service), &settings).unwrap()
    }

    async fn response_json(response: Response<Body>) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    const LEGACY_EXAMPLE_ECHO_PATH: &str = "/api/v1/examples/echo";

    fn api_uri(path: &str) -> String {
        format!("{API_PREFIX}{path}")
    }

    #[tokio::test]
    async fn live_endpoint_returns_success() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["status"], "ok");
        assert!(body.get("code").is_none());
    }

    #[tokio::test]
    async fn ready_endpoint_returns_service_unavailable_when_dependency_fails() {
        let app = build_test_app(Err(AppError::dependency_unavailable(
            "postgres readiness check failed",
        )));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .header("x-request-id", "test-request-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = response_json(response).await;
        assert_eq!(body["code"], "dependency_unavailable");
        assert_eq!(body["request_id"], "test-request-id");
    }

    #[tokio::test]
    async fn ready_endpoint_returns_success_when_dependencies_are_ready() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["checks"]["postgres"], "ok");
        assert_eq!(body["checks"]["redis"], "ok");
        assert!(body.get("code").is_none());
    }

    #[tokio::test]
    async fn fallback_returns_not_found_envelope() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/missing")
                    .header("x-request-id", "req-404")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response_json(response).await;
        assert_eq!(body["code"], "not_found");
        assert_eq!(body["message"], "route not found");
        assert_eq!(body["request_id"], "req-404");
    }

    #[tokio::test]
    async fn health_router_returns_method_not_allowed_envelope() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/health/live")
                    .header("x-request-id", "req-405")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        let body = response_json(response).await;
        assert_eq!(body["code"], "method_not_allowed");
        assert_eq!(body["request_id"], "req-405");
    }

    #[tokio::test]
    async fn invalid_json_syntax_returns_bad_request() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(api_uri(EXAMPLE_ECHO_PATH))
                    .header("content-type", "application/json")
                    .header("x-request-id", "req-json-syntax")
                    .body(Body::from("{\"name\":"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;
        assert_eq!(body["code"], "invalid_request");
        assert_eq!(body["request_id"], "req-json-syntax");
        assert_eq!(body["details"]["fields"][0]["location"], "body");
        assert_eq!(body["details"]["fields"][0]["reason"], "json_syntax");
    }

    #[tokio::test]
    async fn json_type_mismatch_returns_validation_error() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(api_uri(EXAMPLE_ECHO_PATH))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"noodles","quantity":"many"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = response_json(response).await;
        assert_eq!(body["code"], "validation_error");
        assert_eq!(body["details"]["fields"][0]["location"], "body");
        assert_eq!(body["details"]["fields"][0]["field"], "quantity");
    }

    #[tokio::test]
    async fn query_parse_failure_returns_bad_request() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("{}?page=abc", api_uri(EXAMPLE_SEARCH_PATH)))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;
        assert_eq!(body["code"], "invalid_request");
        assert_eq!(body["details"]["fields"][0]["location"], "query");
        assert_eq!(body["details"]["fields"][0]["field"], "page");
    }

    #[tokio::test]
    async fn path_parse_failure_returns_bad_request() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(api_uri(
                        &EXAMPLE_ITEM_PATH.replace("{item_id}", "not-a-number"),
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response_json(response).await;
        assert_eq!(body["code"], "invalid_request");
        assert_eq!(body["details"]["fields"][0]["location"], "path");
        assert_eq!(body["details"]["fields"][0]["field"], "item_id");
    }

    #[tokio::test]
    async fn missing_json_content_type_returns_unsupported_media_type() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(api_uri(EXAMPLE_ECHO_PATH))
                    .body(Body::from(r#"{"name":"rice","quantity":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let body = response_json(response).await;
        assert_eq!(body["code"], "unsupported_media_type");
    }

    #[tokio::test]
    async fn oversized_json_body_returns_payload_too_large() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));
        let large_name = "a".repeat(1024 * 1024);
        let payload = format!(r#"{{"name":"{large_name}","quantity":1}}"#);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(api_uri(EXAMPLE_ECHO_PATH))
                    .header("content-type", "application/json")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let body = response_json(response).await;
        assert_eq!(body["code"], "payload_too_large");
    }

    #[tokio::test]
    async fn openapi_endpoint_returns_json_document_with_error_schemas() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        let json = body.to_string();
        let expected_path = api_uri(EXAMPLE_ECHO_PATH);

        assert!(json.contains("FieldIssue"));
        assert!(json.contains("ErrorDetails"));
        assert!(json.contains(&expected_path));
        assert!(!json.contains(LEGACY_EXAMPLE_ECHO_PATH));
    }

    #[tokio::test]
    async fn old_api_v1_path_returns_not_found() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(LEGACY_EXAMPLE_ECHO_PATH)
                    .header("x-request-id", "req-old-api")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response_json(response).await;
        assert_eq!(body["code"], "not_found");
        assert_eq!(body["request_id"], "req-old-api");
    }

    #[tokio::test]
    async fn docs_endpoint_is_accessible() {
        let app = build_test_app(Ok(DependencyChecks {
            postgres: "ok".to_string(),
            redis: "ok".to_string(),
        }));

        let response = app
            .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert!(response.status().is_success() || response.status().is_redirection());
    }
}
