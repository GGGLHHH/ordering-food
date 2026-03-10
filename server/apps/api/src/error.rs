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
use ts_rs::TS;
use utoipa::ToSchema;

type BoxError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<ErrorDetails>,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct ErrorDetails {
    pub fields: Vec<FieldIssue>,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct FieldIssue {
    pub location: FieldLocation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub field: Option<String>,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum FieldLocation {
    Body,
    Query,
    Path,
}

#[derive(Debug, Clone, Copy)]
enum AppErrorKind {
    InvalidRequest,
    ValidationError,
    Conflict,
    UnsupportedMediaType,
    PayloadTooLarge,
    NotFound,
    MethodNotAllowed,
    DependencyUnavailable,
    Internal,
}

impl AppErrorKind {
    fn status(self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::ValidationError => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict => StatusCode::CONFLICT,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::DependencyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::ValidationError => "validation_error",
            Self::Conflict => "conflict",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::PayloadTooLarge => "payload_too_large",
            Self::NotFound => "not_found",
            Self::MethodNotAllowed => "method_not_allowed",
            Self::DependencyUnavailable => "dependency_unavailable",
            Self::Internal => "internal_error",
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
    details: Option<ErrorDetails>,
    span_trace: SpanTrace,
}

impl AppError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::InvalidRequest, message)
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::ValidationError, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Conflict, message)
    }

    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::UnsupportedMediaType, message)
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::PayloadTooLarge, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::NotFound, message)
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::MethodNotAllowed, message)
    }

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

    pub fn with_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn with_details(mut self, details: ErrorDetails) -> Self {
        self.details = Some(details);
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
            details: None,
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
            details: self.details.clone(),
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
                error_details = ?self.details,
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
                error_details = ?self.details,
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

    #[tokio::test]
    async fn response_serializes_error_details() {
        let response = AppError::validation_error("request body failed validation")
            .with_request_id(Some("request-123".to_string()))
            .with_details(ErrorDetails {
                fields: vec![FieldIssue {
                    location: FieldLocation::Body,
                    field: Some("quantity".to_string()),
                    reason: "invalid_type".to_string(),
                    message: "invalid type: string \"a lot\", expected u32".to_string(),
                }],
            })
            .into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("validation_error"));
        assert!(body.contains("quantity"));
        assert!(body.contains("invalid_type"));
    }
}
