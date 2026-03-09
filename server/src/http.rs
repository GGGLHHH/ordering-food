use crate::error::{AppError, ErrorDetails, FieldIssue, FieldLocation};
use axum::{
    Json,
    extract::rejection::{
        BytesRejection, FailedToBufferBody, JsonRejection, PathRejection, QueryRejection,
    },
    extract::{FromRequest, FromRequestParts, MatchedPath, Path, Query, Request},
    http::request::Parts,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{convert::Infallible, error::Error as StdError, ops::Deref};
use tower_http::request_id::RequestId;
use utoipa::ToSchema;

pub const API_BODY_LIMIT_BYTES: usize = 1024 * 1024;

type JsonPathError = serde_path_to_error::Error<serde_json::Error>;
type QueryPathError = serde_path_to_error::Error<serde_urlencoded::de::Error>;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: Option<String>,
    pub matched_path: String,
}

impl RequestContext {
    pub fn from_parts(parts: &Parts) -> Self {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .and_then(|request_id| request_id.header_value().to_str().ok())
            .map(ToOwned::to_owned);

        let matched_path = parts
            .extensions
            .get::<MatchedPath>()
            .map(MatchedPath::as_str)
            .unwrap_or_else(|| parts.uri.path())
            .to_string();

        Self {
            request_id,
            matched_path,
        }
    }
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_parts(parts))
    }
}

#[derive(Debug, Clone)]
pub struct ApiJson<T>(pub T);

impl<T> ApiJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ApiJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let context = RequestContext::from_parts(&parts);
        let req = Request::from_parts(parts, body);

        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(map_json_rejection(rejection, &context)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiQuery<T>(pub T);

impl<T> ApiQuery<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ApiQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let context = RequestContext::from_parts(parts);

        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(value)) => Ok(Self(value)),
            Err(rejection) => Err(map_query_rejection(rejection, &context)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiPath<T>(pub T);

impl<T> ApiPath<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ApiPath<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let context = RequestContext::from_parts(parts);

        match Path::<T>::from_request_parts(parts, state).await {
            Ok(Path(value)) => Ok(Self(value)),
            Err(rejection) => Err(map_path_rejection(rejection, &context)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageMeta {
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
}

pub async fn not_found(context: RequestContext) -> AppError {
    let _ = &context.matched_path;
    AppError::not_found("route not found").with_request_id(context.request_id)
}

pub async fn method_not_allowed(context: RequestContext) -> AppError {
    let _ = &context.matched_path;
    AppError::method_not_allowed("method not allowed").with_request_id(context.request_id)
}

fn map_json_rejection(rejection: JsonRejection, context: &RequestContext) -> AppError {
    let error =
        match rejection {
            JsonRejection::JsonSyntaxError(err) => {
                AppError::invalid_request("request body contains invalid JSON syntax").with_details(
                    single_issue(FieldLocation::Body, None, "json_syntax", err.body_text()),
                )
            }
            JsonRejection::JsonDataError(err) => {
                let source: Option<&JsonPathError> = find_source(&err);
                let field = source.and_then(|source| path_to_field(source.path().to_string()));
                let reason = source
                    .map(|source| classify_deserialize_message(&source.inner().to_string()))
                    .unwrap_or("deserialize_error");
                let message = source
                    .map(|source| source.inner().to_string())
                    .unwrap_or_else(|| err.body_text());

                AppError::validation_error("request body failed validation")
                    .with_details(single_issue(FieldLocation::Body, field, reason, message))
            }
            JsonRejection::MissingJsonContentType(_) => {
                AppError::unsupported_media_type("request content type must be application/json")
            }
            JsonRejection::BytesRejection(rejection) => map_bytes_rejection(rejection),
            _ => AppError::internal("unexpected JSON extractor rejection"),
        };

    error.with_request_id(context.request_id.clone())
}

fn map_query_rejection(rejection: QueryRejection, context: &RequestContext) -> AppError {
    let error =
        match rejection {
            QueryRejection::FailedToDeserializeQueryString(err) => {
                let source: Option<&QueryPathError> = find_source(&err);
                let field = source.and_then(|source| path_to_field(source.path().to_string()));
                let reason = source
                    .map(|source| classify_deserialize_message(&source.inner().to_string()))
                    .unwrap_or("deserialize_error");
                let message = source
                    .map(|source| source.inner().to_string())
                    .unwrap_or_else(|| err.body_text());

                AppError::invalid_request("query parameters are invalid")
                    .with_details(single_issue(FieldLocation::Query, field, reason, message))
            }
            _ => AppError::internal("unexpected query extractor rejection"),
        };

    error.with_request_id(context.request_id.clone())
}

fn map_path_rejection(rejection: PathRejection, context: &RequestContext) -> AppError {
    let error = match rejection {
        PathRejection::FailedToDeserializePathParams(err) => {
            use axum::extract::path::ErrorKind;

            match err.into_kind() {
                ErrorKind::InvalidUtf8InPathParam { key } => AppError::invalid_request(
                    "path parameters are invalid",
                )
                .with_details(single_issue(
                    FieldLocation::Path,
                    Some(key),
                    "invalid_utf8",
                    "path parameter contains invalid UTF-8",
                )),
                ErrorKind::ParseErrorAtKey {
                    key,
                    value,
                    expected_type,
                } => AppError::invalid_request("path parameters are invalid").with_details(
                    single_issue(
                        FieldLocation::Path,
                        Some(key),
                        "parse_error",
                        format!("cannot parse `{value}` as `{expected_type}`"),
                    ),
                ),
                ErrorKind::DeserializeError {
                    key,
                    value,
                    message,
                } => AppError::invalid_request("path parameters are invalid").with_details(
                    single_issue(
                        FieldLocation::Path,
                        Some(key),
                        classify_deserialize_message(&message),
                        format!("cannot parse `{value}`: {message}"),
                    ),
                ),
                ErrorKind::ParseError {
                    value,
                    expected_type,
                } => AppError::invalid_request("path parameters are invalid").with_details(
                    single_issue(
                        FieldLocation::Path,
                        None,
                        "parse_error",
                        format!("cannot parse `{value}` as `{expected_type}`"),
                    ),
                ),
                ErrorKind::ParseErrorAtIndex {
                    index,
                    value,
                    expected_type,
                } => AppError::invalid_request("path parameters are invalid").with_details(
                    single_issue(
                        FieldLocation::Path,
                        Some(index.to_string()),
                        "parse_error",
                        format!("cannot parse `{value}` as `{expected_type}`"),
                    ),
                ),
                ErrorKind::Message(message) => {
                    AppError::invalid_request("path parameters are invalid").with_details(
                        single_issue(FieldLocation::Path, None, "deserialize_error", message),
                    )
                }
                ErrorKind::WrongNumberOfParameters { .. } | ErrorKind::UnsupportedType { .. } => {
                    AppError::internal("path extractor is misconfigured")
                }
                _ => AppError::internal("unexpected path extractor rejection"),
            }
        }
        PathRejection::MissingPathParams(_) => {
            AppError::internal("path extractor is misconfigured")
        }
        _ => AppError::internal("unexpected path extractor rejection"),
    };

    error.with_request_id(context.request_id.clone())
}

fn map_bytes_rejection(rejection: BytesRejection) -> AppError {
    match rejection {
        BytesRejection::FailedToBufferBody(FailedToBufferBody::LengthLimitError(_)) => {
            AppError::payload_too_large("request body exceeds the allowed size limit")
        }
        BytesRejection::FailedToBufferBody(FailedToBufferBody::UnknownBodyError(_)) => {
            AppError::invalid_request("request body could not be read")
        }
        _ => AppError::internal("unexpected request body rejection"),
    }
}

fn single_issue(
    location: FieldLocation,
    field: Option<String>,
    reason: impl Into<String>,
    message: impl Into<String>,
) -> ErrorDetails {
    ErrorDetails {
        fields: vec![FieldIssue {
            location,
            field,
            reason: reason.into(),
            message: message.into(),
        }],
    }
}

fn classify_deserialize_message(message: &str) -> &'static str {
    let message = message.to_ascii_lowercase();

    if message.starts_with("invalid type") {
        "invalid_type"
    } else if message.starts_with("invalid value") {
        "invalid_value"
    } else if message.starts_with("missing field") {
        "missing_field"
    } else if message.starts_with("unknown field") {
        "unknown_field"
    } else if message.starts_with("unknown variant") {
        "unknown_variant"
    } else {
        "deserialize_error"
    }
}

fn path_to_field(path: String) -> Option<String> {
    let path = path.trim().trim_start_matches('.').to_string();

    if path.is_empty() { None } else { Some(path) }
}

fn find_source<'a, T>(error: &'a (dyn StdError + 'static)) -> Option<&'a T>
where
    T: StdError + 'static,
{
    let mut current = Some(error);

    while let Some(error) = current {
        if let Some(found) = error.downcast_ref::<T>() {
            return Some(found);
        }

        current = error.source();
    }

    None
}
