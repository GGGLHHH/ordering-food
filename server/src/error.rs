use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::error::Error as StdError;
use thiserror::Error;
use tracing::{error, warn};
use tracing_error::SpanTrace;
use utoipa::ToSchema;

type BoxError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum AppErrorKind {
    DependencyUnavailable,
    Internal,
    Validation,
    NotFound,
}

impl AppErrorKind {
    fn status(self) -> StatusCode {
        match self {
            Self::DependencyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::DependencyUnavailable => "dependency_unavailable",
            Self::Internal => "internal_error",
            Self::Validation => "validation_error",
            Self::NotFound => "not_found",
        }
    }
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct AppError {
    kind: AppErrorKind,
    message: String,
    #[source]
    source: Option<BoxError>,
    request_id: Option<String>,
    span_trace: SpanTrace,
}

impl AppError {
    pub fn dependency_unavailable(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::DependencyUnavailable, message)
    }

    pub fn dependency_unavailable_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::dependency_unavailable(message).with_source(source)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Internal, message)
    }

    pub fn internal_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::internal(message).with_source(source)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Validation, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::NotFound, message)
    }

    pub fn with_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn status(&self) -> StatusCode {
        self.kind.status()
    }

    fn code(&self) -> &'static str {
        self.kind.code()
    }

    fn new(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
            request_id: None,
            span_trace: SpanTrace::capture(),
        }
    }

    fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    fn to_envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope {
            code: self.code().to_string(),
            message: self.message.clone(),
            request_id: self.request_id.clone(),
        }
    }

    fn log(&self) {
        let request_id = self.request_id.as_deref().unwrap_or("-");
        let status_code = self.status().as_u16();
        let error_code = self.code();
        let error_chain = self
            .source
            .as_deref()
            .map(format_error_chain)
            .unwrap_or_else(|| self.message.clone());

        if self.status().is_server_error() {
            error!(
                status_code,
                error_code,
                request_id,
                error_message = %self.message,
                error_chain = %error_chain,
                span_trace = %self.span_trace,
                "request returned error response"
            );
        } else {
            warn!(
                status_code,
                error_code,
                request_id,
                error_message = %self.message,
                error_chain = %error_chain,
                "request returned client error response"
            );
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.log();
        (self.status(), Json(self.to_envelope())).into_response()
    }
}

fn format_error_chain(error: &(dyn StdError + Send + Sync + 'static)) -> String {
    let mut chain = vec![error.to_string()];
    let mut current = error.source();

    while let Some(source) = current {
        chain.push(source.to_string());
        current = source.source();
    }

    chain.join(": ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use std::io;

    #[tokio::test]
    async fn response_does_not_expose_internal_source_details() {
        let response = AppError::internal_with_source(
            "internal server error",
            io::Error::other("sensitive internal detail"),
        )
        .with_request_id(Some("request-123".to_string()))
        .into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("internal_error"));
        assert!(body.contains("internal server error"));
        assert!(body.contains("request-123"));
        assert!(!body.contains("sensitive internal detail"));
    }
}
