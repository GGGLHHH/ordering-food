use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    http::{self, ApiPath, AuthenticatedSubject, RequestContext},
};
use axum::{Extension, Json, Router, extract::DefaultBodyLimit, routing::post};
use ordering_food_access_published::OrderManagementAccessGateway;
use ordering_food_fulfillment_application::{
    AcceptOrderInput, ApplicationError as FulfillmentApplicationError,
    CommercialOrderProjectionQueryService, CommercialOrderProjectionReadModel, CompleteOrderInput,
    FulfillmentModule, MarkOrderReadyForPickupInput, RejectOrderByStoreInput,
    StartPreparingOrderInput, WorkflowOrderQueryService,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use ts_rs::TS;
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const ORDER_ROUTE_PREFIX: &str = "/api/orders";
pub(crate) const ORDER_ACCEPT_PATH: &str = "/api/orders/{order_id}/accept";
pub(crate) const ORDER_START_PREPARING_PATH: &str = "/api/orders/{order_id}/start-preparing";
pub(crate) const ORDER_READY_PATH: &str = "/api/orders/{order_id}/ready";
pub(crate) const ORDER_COMPLETE_PATH: &str = "/api/orders/{order_id}/complete";
pub(crate) const ORDER_REJECT_PATH: &str = "/api/orders/{order_id}/reject";

const ACCEPT_ROUTE_PATH: &str = "/{order_id}/accept";
const START_PREPARING_ROUTE_PATH: &str = "/{order_id}/start-preparing";
const READY_ROUTE_PATH: &str = "/{order_id}/ready";
const COMPLETE_ROUTE_PATH: &str = "/{order_id}/complete";
const REJECT_ROUTE_PATH: &str = "/{order_id}/reject";

pub fn router(module: Arc<FulfillmentModule>) -> Router<AppState> {
    Router::new()
        .route(ACCEPT_ROUTE_PATH, post(accept_order))
        .route(START_PREPARING_ROUTE_PATH, post(start_preparing_order))
        .route(READY_ROUTE_PATH, post(mark_order_ready))
        .route(COMPLETE_ROUTE_PATH, post(complete_order))
        .route(REJECT_ROUTE_PATH, post(reject_order))
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(DefaultBodyLimit::max(http::API_BODY_LIMIT_BYTES))
        .layer(Extension(module))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        accept_order,
        start_preparing_order,
        mark_order_ready,
        complete_order,
        reject_order,
    ),
    components(
        schemas(
            ErrorEnvelope,
            FulfillmentOrderPath,
            FulfillmentOrderItemResponse,
            FulfillmentOrderResponse,
        )
    ),
    tags(
        (name = "fulfillment", description = "Store-facing workflow endpoints")
    )
)]
pub struct FulfillmentApiDoc;

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema, TS)]
#[into_params(parameter_in = Path)]
pub struct FulfillmentOrderPath {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct FulfillmentOrderItemResponse {
    pub line_number: i32,
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct FulfillmentOrderResponse {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub commercial_status: String,
    pub workflow_status: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub commercial_created_at: String,
    pub commercial_updated_at: String,
    pub workflow_created_at: String,
    pub workflow_updated_at: String,
    pub items: Vec<FulfillmentOrderItemResponse>,
}

#[utoipa::path(
    post,
    path = ORDER_ACCEPT_PATH,
    tag = "fulfillment",
    params(FulfillmentOrderPath),
    responses(
        (status = 200, description = "Accept order", body = FulfillmentOrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be accepted", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn accept_order(
    Extension(module): Extension<Arc<FulfillmentModule>>,
    Extension(access): Extension<Arc<dyn OrderManagementAccessGateway>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<FulfillmentOrderPath>,
) -> Result<Json<FulfillmentOrderResponse>, AppError> {
    authorize_store_action(
        &module.workflow_queries(),
        access.as_ref(),
        &path.order_id,
        &subject.subject_id,
        context.request_id.clone(),
    )
    .await?;

    module
        .accept_order()
        .execute(AcceptOrderInput {
            order_id: path.order_id.clone(),
            actor_user_id: subject.subject_id,
        })
        .await
        .map_err(|error| map_fulfillment_error(error, context.request_id.clone()))?;

    let response = load_fulfillment_order_response(
        module.commercial_queries().as_ref(),
        &module.workflow_queries(),
        &path.order_id,
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_START_PREPARING_PATH,
    tag = "fulfillment",
    params(FulfillmentOrderPath),
    responses(
        (status = 200, description = "Start preparing order", body = FulfillmentOrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot start preparing", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn start_preparing_order(
    Extension(module): Extension<Arc<FulfillmentModule>>,
    Extension(access): Extension<Arc<dyn OrderManagementAccessGateway>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<FulfillmentOrderPath>,
) -> Result<Json<FulfillmentOrderResponse>, AppError> {
    authorize_store_action(
        &module.workflow_queries(),
        access.as_ref(),
        &path.order_id,
        &subject.subject_id,
        context.request_id.clone(),
    )
    .await?;

    module
        .start_preparing_order()
        .execute(StartPreparingOrderInput {
            order_id: path.order_id.clone(),
            actor_user_id: subject.subject_id,
        })
        .await
        .map_err(|error| map_fulfillment_error(error, context.request_id.clone()))?;

    let response = load_fulfillment_order_response(
        module.commercial_queries().as_ref(),
        &module.workflow_queries(),
        &path.order_id,
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_READY_PATH,
    tag = "fulfillment",
    params(FulfillmentOrderPath),
    responses(
        (status = 200, description = "Mark order ready for pickup", body = FulfillmentOrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be marked ready", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn mark_order_ready(
    Extension(module): Extension<Arc<FulfillmentModule>>,
    Extension(access): Extension<Arc<dyn OrderManagementAccessGateway>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<FulfillmentOrderPath>,
) -> Result<Json<FulfillmentOrderResponse>, AppError> {
    authorize_store_action(
        &module.workflow_queries(),
        access.as_ref(),
        &path.order_id,
        &subject.subject_id,
        context.request_id.clone(),
    )
    .await?;

    module
        .mark_order_ready_for_pickup()
        .execute(MarkOrderReadyForPickupInput {
            order_id: path.order_id.clone(),
            actor_user_id: subject.subject_id,
        })
        .await
        .map_err(|error| map_fulfillment_error(error, context.request_id.clone()))?;

    let response = load_fulfillment_order_response(
        module.commercial_queries().as_ref(),
        &module.workflow_queries(),
        &path.order_id,
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_COMPLETE_PATH,
    tag = "fulfillment",
    params(FulfillmentOrderPath),
    responses(
        (status = 200, description = "Complete order", body = FulfillmentOrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be completed", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn complete_order(
    Extension(module): Extension<Arc<FulfillmentModule>>,
    Extension(access): Extension<Arc<dyn OrderManagementAccessGateway>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<FulfillmentOrderPath>,
) -> Result<Json<FulfillmentOrderResponse>, AppError> {
    authorize_store_action(
        &module.workflow_queries(),
        access.as_ref(),
        &path.order_id,
        &subject.subject_id,
        context.request_id.clone(),
    )
    .await?;

    module
        .complete_order()
        .execute(CompleteOrderInput {
            order_id: path.order_id.clone(),
            actor_user_id: subject.subject_id,
        })
        .await
        .map_err(|error| map_fulfillment_error(error, context.request_id.clone()))?;

    let response = load_fulfillment_order_response(
        module.commercial_queries().as_ref(),
        &module.workflow_queries(),
        &path.order_id,
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_REJECT_PATH,
    tag = "fulfillment",
    params(FulfillmentOrderPath),
    responses(
        (status = 200, description = "Reject order", body = FulfillmentOrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be rejected", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn reject_order(
    Extension(module): Extension<Arc<FulfillmentModule>>,
    Extension(access): Extension<Arc<dyn OrderManagementAccessGateway>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<FulfillmentOrderPath>,
) -> Result<Json<FulfillmentOrderResponse>, AppError> {
    authorize_store_action(
        &module.workflow_queries(),
        access.as_ref(),
        &path.order_id,
        &subject.subject_id,
        context.request_id.clone(),
    )
    .await?;

    module
        .reject_order_by_store()
        .execute(RejectOrderByStoreInput {
            order_id: path.order_id.clone(),
            actor_user_id: subject.subject_id,
        })
        .await
        .map_err(|error| map_fulfillment_error(error, context.request_id.clone()))?;

    let response = load_fulfillment_order_response(
        module.commercial_queries().as_ref(),
        &module.workflow_queries(),
        &path.order_id,
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

async fn authorize_store_action(
    workflow_queries: &WorkflowOrderQueryService,
    access: &dyn OrderManagementAccessGateway,
    order_id: &str,
    subject_id: &str,
    request_id: Option<String>,
) -> Result<(), AppError> {
    let workflow = workflow_queries
        .get_by_ordering_order_id(order_id)
        .await
        .map_err(|error| map_fulfillment_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("order was not found").with_request_id(request_id.clone())
        })?;

    let authorized = access
        .can_manage_order(subject_id, &workflow.store_id)
        .await
        .map_err(|_| {
            AppError::internal("internal server error").with_request_id(request_id.clone())
        })?;

    if !authorized {
        return Err(AppError::not_found("order was not found").with_request_id(request_id));
    }

    Ok(())
}

async fn load_fulfillment_order_response(
    commercial_queries: &CommercialOrderProjectionQueryService,
    workflow_queries: &WorkflowOrderQueryService,
    order_id: &str,
    request_id: Option<String>,
) -> Result<FulfillmentOrderResponse, AppError> {
    let workflow = workflow_queries
        .get_by_ordering_order_id(order_id)
        .await
        .map_err(|error| map_fulfillment_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("order was not found").with_request_id(request_id.clone())
        })?;

    let commercial_order = commercial_queries
        .get_by_ordering_order_id(order_id)
        .await
        .map_err(|error| map_fulfillment_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("order was not found").with_request_id(request_id.clone())
        })?;

    map_fulfillment_order_response(commercial_order, workflow)
        .map_err(|error| error.with_request_id(request_id))
}

#[allow(clippy::result_large_err)]
fn map_fulfillment_order_response(
    commercial_order: CommercialOrderProjectionReadModel,
    workflow: ordering_food_fulfillment_application::WorkflowOrderReadModel,
) -> Result<FulfillmentOrderResponse, AppError> {
    Ok(FulfillmentOrderResponse {
        order_id: commercial_order.order_id,
        customer_id: commercial_order.customer_id,
        store_id: commercial_order.store_id,
        commercial_status: commercial_order.status,
        workflow_status: workflow.status,
        subtotal_amount: commercial_order.subtotal_amount,
        total_amount: commercial_order.total_amount,
        commercial_created_at: format_timestamp(commercial_order.created_at)?,
        commercial_updated_at: format_timestamp(commercial_order.updated_at)?,
        workflow_created_at: format_timestamp(workflow.created_at)?,
        workflow_updated_at: format_timestamp(workflow.updated_at)?,
        items: commercial_order
            .items
            .into_iter()
            .map(|item| FulfillmentOrderItemResponse {
                line_number: item.line_number,
                catalog_item_id: item.catalog_item_id,
                name: item.name,
                unit_price_amount: item.unit_price_amount,
                quantity: item.quantity,
                line_total_amount: item.line_total_amount,
            })
            .collect(),
    })
}

#[allow(clippy::result_large_err)]
fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, AppError> {
    timestamp
        .format(&Rfc3339)
        .map_err(|error| AppError::internal_with_source("internal server error", error))
}

fn map_fulfillment_error(
    error: FulfillmentApplicationError,
    request_id: Option<String>,
) -> AppError {
    match error {
        FulfillmentApplicationError::Validation { message } => {
            AppError::validation_error(message).with_request_id(request_id)
        }
        FulfillmentApplicationError::NotFound { message } => {
            AppError::not_found(message).with_request_id(request_id)
        }
        FulfillmentApplicationError::Conflict { message } => {
            AppError::conflict(message).with_request_id(request_id)
        }
        FulfillmentApplicationError::Unexpected { .. } => {
            AppError::internal("internal server error").with_request_id(request_id)
        }
    }
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
        Extension,
        body::{Body, to_bytes},
        http::{HeaderName, Request, StatusCode},
        response::Response,
    };
    use ordering_food_access_published::{AccessCollaborationError, OrderManagementAccessGateway};
    use ordering_food_fulfillment_application::{
        CommercialOrderProjectionItemReadModel, CommercialOrderProjectionReadModel,
        CommercialOrderProjectionReadRepository, CommercialOrderProjectionStore,
        TransactionContext as FulfillmentTransactionContext,
        TransactionManager as FulfillmentTransactionManager, WorkflowOrderReadRepository,
        WorkflowOrderRepository,
    };
    use ordering_food_fulfillment_domain::FulfillmentOrder;
    use ordering_food_identity_published::{
        AccessTokenVerifier, AuthenticatedSubjectRef, IdentityCollaborationError,
    };
    use ordering_food_shared_kernel::Timestamp;
    use serde_json::Value;
    use std::{
        any::Any,
        collections::{HashMap, HashSet},
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;
    use tower::ServiceExt;
    use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

    struct StubReadiness;

    #[async_trait]
    impl ReadinessProbe for StubReadiness {
        async fn check(&self) -> Result<DependencyChecks, AppError> {
            Ok(DependencyChecks::ok("ok", "ok"))
        }
    }

    struct FakeTokenVerifier;

    impl AccessTokenVerifier for FakeTokenVerifier {
        fn verify_access_token(
            &self,
            token: &str,
        ) -> Result<AuthenticatedSubjectRef, IdentityCollaborationError> {
            let user_id = token
                .strip_prefix("token-")
                .ok_or_else(|| IdentityCollaborationError::new("invalid token"))?
                .to_string();

            Ok(AuthenticatedSubjectRef::new(user_id))
        }
    }

    #[derive(Default)]
    struct FakeWorkflowTransactionContext;

    impl FulfillmentTransactionContext for FakeWorkflowTransactionContext {
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
            self
        }
    }

    #[derive(Default)]
    struct FakeWorkflowTransactionManager;

    #[async_trait]
    impl FulfillmentTransactionManager for FakeWorkflowTransactionManager {
        async fn begin(
            &self,
        ) -> Result<Box<dyn FulfillmentTransactionContext>, FulfillmentApplicationError> {
            Ok(Box::new(FakeWorkflowTransactionContext))
        }

        async fn commit(
            &self,
            _tx: Box<dyn FulfillmentTransactionContext>,
        ) -> Result<(), FulfillmentApplicationError> {
            Ok(())
        }

        async fn rollback(
            &self,
            _tx: Box<dyn FulfillmentTransactionContext>,
        ) -> Result<(), FulfillmentApplicationError> {
            Ok(())
        }
    }

    struct FixedClock {
        now: Timestamp,
    }

    impl ordering_food_fulfillment_application::Clock for FixedClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FixedIdGenerator;

    impl ordering_food_fulfillment_application::IdGenerator for FixedIdGenerator {
        fn next_fulfillment_order_id(
            &self,
        ) -> ordering_food_fulfillment_domain::FulfillmentOrderId {
            ordering_food_fulfillment_domain::FulfillmentOrderId::new("workflow-generated")
        }
    }

    #[derive(Clone, Default)]
    struct InMemoryWorkflowRepository {
        orders: Arc<Mutex<HashMap<String, FulfillmentOrder>>>,
    }

    impl InMemoryWorkflowRepository {
        fn seed(&self, order: FulfillmentOrder) {
            self.orders
                .lock()
                .unwrap()
                .insert(order.ordering_order_id().to_string(), order);
        }
    }

    #[async_trait]
    impl WorkflowOrderRepository for InMemoryWorkflowRepository {
        async fn find_by_ordering_order_id(
            &self,
            _tx: &mut dyn FulfillmentTransactionContext,
            ordering_order_id: &str,
        ) -> Result<Option<FulfillmentOrder>, FulfillmentApplicationError> {
            Ok(self.orders.lock().unwrap().get(ordering_order_id).cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn FulfillmentTransactionContext,
            order: &FulfillmentOrder,
        ) -> Result<(), FulfillmentApplicationError> {
            self.seed(order.clone());
            Ok(())
        }

        async fn update(
            &self,
            _tx: &mut dyn FulfillmentTransactionContext,
            order: &FulfillmentOrder,
        ) -> Result<(), FulfillmentApplicationError> {
            self.seed(order.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl WorkflowOrderReadRepository for InMemoryWorkflowRepository {
        async fn get_by_ordering_order_id(
            &self,
            ordering_order_id: &str,
        ) -> Result<
            Option<ordering_food_fulfillment_application::WorkflowOrderReadModel>,
            FulfillmentApplicationError,
        > {
            Ok(self
                .orders
                .lock()
                .unwrap()
                .get(ordering_order_id)
                .map(
                    |order| ordering_food_fulfillment_application::WorkflowOrderReadModel {
                        fulfillment_order_id: order.id().as_str().to_string(),
                        ordering_order_id: order.ordering_order_id().to_string(),
                        store_id: order.store_id().to_string(),
                        status: order.status().as_str().to_string(),
                        created_at: order.created_at(),
                        updated_at: order.updated_at(),
                    },
                ))
        }
    }

    #[derive(Default)]
    struct InMemoryCommercialOrderProjectionRepository {
        orders: Mutex<HashMap<String, CommercialOrderProjectionReadModel>>,
    }

    #[async_trait]
    impl CommercialOrderProjectionReadRepository for InMemoryCommercialOrderProjectionRepository {
        async fn get_by_ordering_order_id(
            &self,
            ordering_order_id: &str,
        ) -> Result<Option<CommercialOrderProjectionReadModel>, FulfillmentApplicationError>
        {
            Ok(self.orders.lock().unwrap().get(ordering_order_id).cloned())
        }
    }

    #[async_trait]
    impl CommercialOrderProjectionStore for InMemoryCommercialOrderProjectionRepository {
        async fn upsert(
            &self,
            _tx: &mut dyn FulfillmentTransactionContext,
            projection: &CommercialOrderProjectionReadModel,
        ) -> Result<(), FulfillmentApplicationError> {
            self.orders
                .lock()
                .unwrap()
                .insert(projection.order_id.clone(), projection.clone());
            Ok(())
        }

        async fn update_status(
            &self,
            _tx: &mut dyn FulfillmentTransactionContext,
            ordering_order_id: &str,
            status: &str,
            updated_at: Timestamp,
        ) -> Result<(), FulfillmentApplicationError> {
            let mut orders = self.orders.lock().unwrap();
            let projection = orders.get_mut(ordering_order_id).ok_or_else(|| {
                FulfillmentApplicationError::not_found("commercial order projection was not found")
            })?;
            projection.status = status.to_string();
            projection.updated_at = updated_at;
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeOrderManagementAccessGateway {
        allowed_pairs: Mutex<HashSet<(String, String)>>,
    }

    #[async_trait]
    impl OrderManagementAccessGateway for FakeOrderManagementAccessGateway {
        async fn can_manage_order(
            &self,
            subject_id: &str,
            store_id: &str,
        ) -> Result<bool, AccessCollaborationError> {
            Ok(self
                .allowed_pairs
                .lock()
                .unwrap()
                .contains(&(subject_id.to_string(), store_id.to_string())))
        }
    }

    impl FakeOrderManagementAccessGateway {
        fn allow(&self, subject_id: &str, store_id: &str) {
            self.allowed_pairs
                .lock()
                .unwrap()
                .insert((subject_id.to_string(), store_id.to_string()));
        }
    }

    fn build_test_app(
        workflow_repository: Arc<InMemoryWorkflowRepository>,
        commercial_order_repository: Arc<InMemoryCommercialOrderProjectionRepository>,
        access_gateway: Arc<FakeOrderManagementAccessGateway>,
    ) -> Router {
        let module = Arc::new(FulfillmentModule::new(
            workflow_repository.clone(),
            workflow_repository,
            commercial_order_repository.clone(),
            commercial_order_repository,
            Arc::new(FakeWorkflowTransactionManager),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:00 UTC),
            }),
            Arc::new(FixedIdGenerator),
        ));
        let request_id_header = HeaderName::from_static("x-request-id");
        let token_verifier: Arc<dyn AccessTokenVerifier> = Arc::new(FakeTokenVerifier);

        Router::new()
            .nest(
                ORDER_ROUTE_PREFIX,
                router(module)
                    .layer(Extension(
                        access_gateway as Arc<dyn OrderManagementAccessGateway>,
                    ))
                    .layer(Extension(token_verifier)),
            )
            .fallback(http::not_found)
            .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
            .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
            .with_state(AppState::new(Arc::new(StubReadiness)))
    }

    async fn response_json(response: Response<Body>) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn seed_order(
        commercial_order_repository: &InMemoryCommercialOrderProjectionRepository,
        workflow_repository: &InMemoryWorkflowRepository,
    ) {
        commercial_order_repository.orders.lock().unwrap().insert(
            "order-1".to_string(),
            CommercialOrderProjectionReadModel {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                status: "placed".to_string(),
                subtotal_amount: 3200,
                total_amount: 3200,
                created_at: datetime!(2026-03-15 09:00 UTC),
                updated_at: datetime!(2026-03-15 09:00 UTC),
                items: vec![CommercialOrderProjectionItemReadModel {
                    line_number: 1,
                    catalog_item_id: "item-1".to_string(),
                    name: "Fried Rice".to_string(),
                    unit_price_amount: 3200,
                    quantity: 1,
                    line_total_amount: 3200,
                }],
            },
        );
        workflow_repository.seed(FulfillmentOrder::bootstrap(
            "workflow-1",
            "order-1",
            "store-1",
            datetime!(2026-03-15 09:00 UTC),
        ));
    }

    #[tokio::test]
    async fn store_staff_can_accept_order_through_fulfillment_route() {
        let workflow_repository = Arc::new(InMemoryWorkflowRepository::default());
        let commercial_order_repository =
            Arc::new(InMemoryCommercialOrderProjectionRepository::default());
        seed_order(&commercial_order_repository, &workflow_repository);
        let access_gateway = Arc::new(FakeOrderManagementAccessGateway::default());
        access_gateway.allow("merchant-1", "store-1");
        let app = build_test_app(
            workflow_repository,
            commercial_order_repository,
            access_gateway,
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/orders/order-1/accept")
                    .header("cookie", "access_token=token-merchant-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["workflow_status"], "accepted");
        assert_eq!(body["commercial_status"], "placed");
    }

    #[tokio::test]
    async fn store_membership_is_required_for_fulfillment_route() {
        let workflow_repository = Arc::new(InMemoryWorkflowRepository::default());
        let commercial_order_repository =
            Arc::new(InMemoryCommercialOrderProjectionRepository::default());
        seed_order(&commercial_order_repository, &workflow_repository);
        let app = build_test_app(
            workflow_repository,
            commercial_order_repository,
            Arc::new(FakeOrderManagementAccessGateway::default()),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/orders/order-1/accept")
                    .header("cookie", "access_token=token-bob")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn fulfillment_route_uses_dedicated_workflow_response_contract() {
        let workflow_repository = Arc::new(InMemoryWorkflowRepository::default());
        let commercial_order_repository =
            Arc::new(InMemoryCommercialOrderProjectionRepository::default());
        seed_order(&commercial_order_repository, &workflow_repository);
        let access_gateway = Arc::new(FakeOrderManagementAccessGateway::default());
        access_gateway.allow("merchant-1", "store-1");
        let app = build_test_app(
            workflow_repository,
            commercial_order_repository,
            access_gateway,
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/orders/order-1/accept")
                    .header("cookie", "access_token=token-merchant-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["workflow_status"], "accepted");
        assert_eq!(body["commercial_status"], "placed");
        assert!(body.get("status").is_none());
    }
}
