use crate::routes::{
    auth::AuthApiDoc,
    catalog::CatalogApiDoc,
    fulfillment::FulfillmentApiDoc,
    identity::IdentityApiDoc,
    ordering::OrderingApiDoc,
    ApiDoc,
};
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as OpenApiDocument;

/// Build the merged OpenAPI document from all context API docs.
///
/// This function statically collects all utoipa `#[derive(OpenApi)]` structs
/// and merges them into a single document. It does not require any runtime
/// infrastructure (no database, no Redis, no server).
pub fn build_merged_openapi_document() -> OpenApiDocument {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(IdentityApiDoc::openapi());
    openapi.merge(AuthApiDoc::openapi());
    openapi.merge(CatalogApiDoc::openapi());
    openapi.merge(OrderingApiDoc::openapi());
    openapi.merge(FulfillmentApiDoc::openapi());
    openapi
}
