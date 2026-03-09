use crate::{
    config::Settings,
    readiness::{ReadinessProbe, RuntimeReadiness},
    routes,
};
use anyhow::{Context, Result, anyhow, ensure};
use axum::{
    Router,
    extract::MatchedPath,
    http::{HeaderName, HeaderValue, Method, Request, Response, header},
};
use redis::aio::MultiplexedConnection;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{sync::Arc, time::Duration};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, RequestId},
    trace::TraceLayer,
};
use tracing::{Span, field, info, info_span, warn};

#[derive(Clone)]
pub struct AppState {
    pub readiness: Arc<dyn ReadinessProbe>,
}

impl AppState {
    pub fn new(readiness: Arc<dyn ReadinessProbe>) -> Self {
        Self { readiness }
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

    let readiness = Arc::new(RuntimeReadiness::new(pg_pool, redis_client));
    let app = build_router(AppState::new(readiness), &settings)?;
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
    use crate::{config::Settings, error::AppError, readiness::DependencyChecks};
    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use std::sync::Mutex;
    use tower::ServiceExt;

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

    fn build_test_app(result: Result<DependencyChecks, AppError>) -> Router {
        let settings = Settings::from_overrides(std::iter::empty::<(String, String)>()).unwrap();
        let readiness = Arc::new(MockReadiness {
            result: Mutex::new(Some(result)),
        });

        build_router(AppState::new(readiness), &settings).unwrap()
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

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("\"status\":\"ok\""));
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

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("dependency_unavailable"));
        assert!(body.contains("test-request-id"));
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

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("\"postgres\":\"ok\""));
        assert!(body.contains("\"redis\":\"ok\""));
    }

    #[tokio::test]
    async fn openapi_endpoint_returns_json_document() {
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

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("\"openapi\""));
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
