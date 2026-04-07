use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    http::{self, ApiJson, ApiPath, AuthenticatedSubject, RequestContext},
};
use axum::{
    Extension, Json, Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use ordering_food_ordering_application::{
    ApplicationError as OrderingApplicationError, CancelOrderByCustomerInput, OrderQueryService,
    OrderingModule, PlaceOrderFromCartInput, PlaceOrderItemInput as ApplicationPlaceOrderItemInput,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use ts_rs::TS;
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const ORDER_ROUTE_PREFIX: &str = "/api/orders";
pub(crate) const ORDERS_PATH: &str = "/api/orders";
pub(crate) const ORDER_PATH: &str = "/api/orders/{order_id}";
pub(crate) const ORDER_CANCEL_PATH: &str = "/api/orders/{order_id}/cancel";

const ROOT_ROUTE_PATH: &str = "/";
const DETAIL_ROUTE_PATH: &str = "/{order_id}";
const CANCEL_ROUTE_PATH: &str = "/{order_id}/cancel";

pub fn router(module: Arc<OrderingModule>) -> Router<AppState> {
    Router::new()
        .route(ROOT_ROUTE_PATH, get(list_orders).post(place_order))
        .route(DETAIL_ROUTE_PATH, get(get_order))
        .route(CANCEL_ROUTE_PATH, post(cancel_order))
        .method_not_allowed_fallback(http::method_not_allowed)
        .layer(DefaultBodyLimit::max(http::API_BODY_LIMIT_BYTES))
        .layer(Extension(module))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        place_order,
        list_orders,
        get_order,
        cancel_order,
    ),
    components(
        schemas(
            ErrorEnvelope,
            PlaceOrderItemRequest,
            PlaceOrderRequest,
            OrderPath,
            OrderItemResponse,
            OrderListItemResponse,
            OrderListResponse,
            OrderResponse,
        )
    ),
    tags(
        (name = "orders", description = "Customer-facing ordering endpoints")
    )
)]
pub struct OrderingApiDoc;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct PlaceOrderItemRequest {
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct PlaceOrderRequest {
    pub store_id: String,
    pub items: Vec<PlaceOrderItemRequest>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema, TS)]
#[into_params(parameter_in = Path)]
pub struct OrderPath {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct OrderItemResponse {
    pub line_number: i32,
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct OrderResponse {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub status: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub created_at: String,
    pub updated_at: String,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct OrderListItemResponse {
    pub order_id: String,
    pub store_id: String,
    pub status: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub item_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, TS)]
pub struct OrderListResponse {
    pub orders: Vec<OrderListItemResponse>,
}

#[utoipa::path(
    post,
    path = ORDERS_PATH,
    tag = "orders",
    request_body = PlaceOrderRequest,
    responses(
        (status = 200, description = "Place a pickup order", body = OrderResponse),
        (status = 400, description = "Invalid request", body = ErrorEnvelope),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 413, description = "Request body exceeds limit", body = ErrorEnvelope),
        (status = 415, description = "Unsupported media type", body = ErrorEnvelope),
        (status = 422, description = "Body validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn place_order(
    Extension(module): Extension<Arc<OrderingModule>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiJson(payload): ApiJson<PlaceOrderRequest>,
) -> Result<Json<OrderResponse>, AppError> {
    let order_id = module
        .place_order_from_cart()
        .execute(PlaceOrderFromCartInput {
            customer_id: subject.subject_id.clone(),
            store_id: payload.store_id,
            items: payload
                .items
                .into_iter()
                .map(|item| ApplicationPlaceOrderItemInput {
                    catalog_item_id: item.catalog_item_id,
                    name: item.name,
                    unit_price_amount: item.unit_price_amount,
                    quantity: item.quantity,
                })
                .collect(),
        })
        .await
        .map_err(|error| map_ordering_error(error, context.request_id.clone()))?;

    let response = load_order_response(
        &module.order_queries(),
        &order_id,
        None,
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = ORDERS_PATH,
    tag = "orders",
    responses(
        (status = 200, description = "List current user's orders", body = OrderListResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
#[allow(clippy::result_large_err)]
pub async fn list_orders(
    Extension(module): Extension<Arc<OrderingModule>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
) -> Result<Json<OrderListResponse>, AppError> {
    let orders = module
        .order_queries()
        .list_by_customer(&subject.subject_id)
        .await
        .map_err(|error| map_ordering_error(error, context.request_id.clone()))?;

    let mut response_orders = Vec::with_capacity(orders.len());
    for order in orders {
        response_orders.push(OrderListItemResponse {
            order_id: order.order_id,
            store_id: order.store_id,
            status: order.status,
            subtotal_amount: order.subtotal_amount,
            total_amount: order.total_amount,
            item_count: order.item_count,
            created_at: format_timestamp(order.created_at)?,
            updated_at: format_timestamp(order.updated_at)?,
        });
    }

    Ok(Json(OrderListResponse {
        orders: response_orders,
    }))
}

#[utoipa::path(
    get,
    path = ORDER_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Get order details", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_order(
    Extension(module): Extension<Arc<OrderingModule>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    let response = load_order_response(
        &module.order_queries(),
        &path.order_id,
        Some(&subject.subject_id),
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_CANCEL_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Cancel order", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order can no longer be cancelled", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn cancel_order(
    Extension(module): Extension<Arc<OrderingModule>>,
    context: RequestContext,
    subject: AuthenticatedSubject,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .cancel_order_by_customer()
        .execute(CancelOrderByCustomerInput {
            order_id: path.order_id.clone(),
            customer_id: subject.subject_id.clone(),
        })
        .await
        .map_err(|error| map_ordering_error(error, context.request_id.clone()))?;

    let response = load_order_response(
        &module.order_queries(),
        &path.order_id,
        Some(&subject.subject_id),
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[allow(clippy::result_large_err)]
pub(crate) async fn load_order_response(
    order_queries: &OrderQueryService,
    order_id: &str,
    customer_id: Option<&str>,
    request_id: Option<String>,
) -> Result<OrderResponse, AppError> {
    let order = order_queries
        .get_by_id(order_id)
        .await
        .map_err(|error| map_ordering_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("order was not found").with_request_id(request_id.clone())
        })?;

    if customer_id.is_some_and(|customer_id| customer_id != order.customer_id) {
        return Err(AppError::not_found("order was not found").with_request_id(request_id));
    }

    map_order_response(order).map_err(|error| error.with_request_id(request_id))
}

#[allow(clippy::result_large_err)]
pub(crate) fn map_order_response(
    order: ordering_food_ordering_application::OrderReadModel,
) -> Result<OrderResponse, AppError> {
    Ok(OrderResponse {
        order_id: order.order_id,
        customer_id: order.customer_id,
        store_id: order.store_id,
        status: order.status,
        subtotal_amount: order.subtotal_amount,
        total_amount: order.total_amount,
        created_at: format_timestamp(order.created_at)?,
        updated_at: format_timestamp(order.updated_at)?,
        items: order
            .items
            .into_iter()
            .map(|item| OrderItemResponse {
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
pub(crate) fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, AppError> {
    timestamp
        .format(&Rfc3339)
        .map_err(|error| AppError::internal_with_source("internal server error", error))
}

pub(crate) fn map_ordering_error(
    error: OrderingApplicationError,
    request_id: Option<String>,
) -> AppError {
    match error {
        OrderingApplicationError::Validation { message } => {
            AppError::validation_error(message).with_request_id(request_id)
        }
        OrderingApplicationError::NotFound { message } => {
            AppError::not_found(message).with_request_id(request_id)
        }
        OrderingApplicationError::Conflict { message } => {
            AppError::conflict(message).with_request_id(request_id)
        }
        OrderingApplicationError::Unexpected { .. } => {
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
    use ordering_food_identity_published::{
        AccessTokenVerifier, AuthenticatedSubjectRef, IdentityCollaborationError,
    };
    use ordering_food_ordering_application::{
        ApplicationError, OrderItemReadModel, OrderListItemReadModel, OrderPlaced,
        OrderReadRepository, OrderRepository, OrderingPublishedEventRecorder, TransactionContext,
        TransactionManager,
    };
    use ordering_food_ordering_domain::{
        CatalogItemId, CustomerId, Order, OrderId, OrderStatus, PlaceOrderItemInput, StoreId,
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

    impl ordering_food_ordering_application::Clock for FakeClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FakeIdGenerator;

    impl ordering_food_ordering_application::IdGenerator for FakeIdGenerator {
        fn next_order_id(&self) -> OrderId {
            OrderId::new("generated-order")
        }
    }

    #[derive(Default)]
    struct InMemoryState {
        orders: HashMap<String, Order>,
    }

    #[derive(Clone, Default)]
    struct InMemoryOrderRepository {
        state: Arc<Mutex<InMemoryState>>,
    }

    impl InMemoryOrderRepository {
        fn seed_order(&self, order: Order) {
            self.state
                .lock()
                .unwrap()
                .orders
                .insert(order.id().as_str().to_string(), order);
        }
    }

    #[async_trait]
    impl OrderRepository for InMemoryOrderRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            order_id: &OrderId,
        ) -> Result<Option<Order>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .orders
                .get(order_id.as_str())
                .cloned())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            order: &Order,
        ) -> Result<(), ApplicationError> {
            self.seed_order(order.clone());
            Ok(())
        }

        async fn update(
            &self,
            _tx: &mut dyn TransactionContext,
            order: &Order,
        ) -> Result<(), ApplicationError> {
            self.seed_order(order.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl OrderReadRepository for InMemoryOrderRepository {
        async fn get_by_id(
            &self,
            order_id: &OrderId,
        ) -> Result<Option<ordering_food_ordering_application::OrderReadModel>, ApplicationError>
        {
            Ok(self
                .state
                .lock()
                .unwrap()
                .orders
                .get(order_id.as_str())
                .map(|order| ordering_food_ordering_application::OrderReadModel {
                    order_id: order.id().as_str().to_string(),
                    customer_id: order.customer_id().as_str().to_string(),
                    store_id: order.store_id().as_str().to_string(),
                    status: order.status().as_str().to_string(),
                    subtotal_amount: order.subtotal_amount(),
                    total_amount: order.total_amount(),
                    created_at: order.created_at(),
                    updated_at: order.updated_at(),
                    items: order
                        .items()
                        .iter()
                        .map(|item| OrderItemReadModel {
                            line_number: item.line_number(),
                            catalog_item_id: item.catalog_item_id().as_str().to_string(),
                            name: item.name().to_string(),
                            unit_price_amount: item.unit_price_amount(),
                            quantity: item.quantity(),
                            line_total_amount: item.line_total_amount(),
                        })
                        .collect(),
                }))
        }

        async fn list_by_customer(
            &self,
            customer_id: &str,
        ) -> Result<Vec<OrderListItemReadModel>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .orders
                .values()
                .filter(|order| order.customer_id().as_str() == customer_id)
                .map(|order| OrderListItemReadModel {
                    order_id: order.id().as_str().to_string(),
                    customer_id: order.customer_id().as_str().to_string(),
                    store_id: order.store_id().as_str().to_string(),
                    status: order.status().as_str().to_string(),
                    subtotal_amount: order.subtotal_amount(),
                    total_amount: order.total_amount(),
                    created_at: order.created_at(),
                    updated_at: order.updated_at(),
                    item_count: order.items().len(),
                })
                .collect())
        }
    }

    #[derive(Default)]
    struct RecordingEventRecorder {
        placed: Mutex<Vec<OrderPlaced>>,
        cancelled: Mutex<Vec<ordering_food_ordering_application::OrderCancelledByCustomer>>,
    }

    #[async_trait]
    impl OrderingPublishedEventRecorder for RecordingEventRecorder {
        async fn record_order_placed(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &OrderPlaced,
        ) -> Result<(), ApplicationError> {
            self.placed.lock().unwrap().push(event.clone());
            Ok(())
        }

        async fn record_order_commercial_state_changed(
            &self,
            _tx: &mut dyn TransactionContext,
            _event: &ordering_food_ordering_application::OrderCommercialStateChanged,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn record_order_cancelled_by_customer(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &ordering_food_ordering_application::OrderCancelledByCustomer,
        ) -> Result<(), ApplicationError> {
            self.cancelled.lock().unwrap().push(event.clone());
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

    fn build_test_app(
        repository: Arc<InMemoryOrderRepository>,
        event_recorder: Arc<RecordingEventRecorder>,
    ) -> Router {
        let module = Arc::new(OrderingModule::new(
            repository.clone(),
            repository,
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-03-15 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
            event_recorder,
        ));
        let request_id_header = HeaderName::from_static("x-request-id");
        let token_verifier: Arc<dyn AccessTokenVerifier> = Arc::new(FakeTokenVerifier);

        Router::new()
            .nest(
                ORDER_ROUTE_PREFIX,
                router(module).layer(Extension(token_verifier)),
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

    fn make_order(order_id: &str, customer_id: &str, status: OrderStatus) -> Order {
        let mut order = Order::place(
            OrderId::new(order_id),
            CustomerId::new(customer_id),
            StoreId::new("store-1"),
            vec![PlaceOrderItemInput {
                catalog_item_id: CatalogItemId::new("item-1"),
                name: "Fried Rice".to_string(),
                unit_price_amount: 3200,
                quantity: 1,
            }],
            datetime!(2026-03-15 09:00 UTC),
        )
        .unwrap();

        if status == OrderStatus::CancelledByCustomer {
            order
                .cancel_by_customer(datetime!(2026-03-15 09:01 UTC))
                .unwrap();
        }

        order
    }

    #[tokio::test]
    async fn place_order_returns_snapshot_and_records_published_event() {
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let app = build_test_app(
            Arc::new(InMemoryOrderRepository::default()),
            event_recorder.clone(),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(ORDERS_PATH)
                    .header("content-type", "application/json")
                    .header("cookie", "access_token=token-customer-1")
                    .body(Body::from(
                        serde_json::json!({
                            "store_id": "store-1",
                            "items": [{
                                "catalog_item_id": "item-1",
                                "name": "Fried Rice",
                                "unit_price_amount": 3200,
                                "quantity": 2
                            }]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["order_id"], "generated-order");
        assert_eq!(body["status"], "placed");
        assert_eq!(body["items"][0]["catalog_item_id"], "item-1");
        assert_eq!(event_recorder.placed.lock().unwrap().len(), 1);
        assert_eq!(
            event_recorder.placed.lock().unwrap()[0].store_id,
            "store-1".to_string()
        );
    }

    #[tokio::test]
    async fn get_order_hides_other_users_order() {
        let repository = Arc::new(InMemoryOrderRepository::default());
        repository.seed_order(make_order("order-1", "customer-1", OrderStatus::Placed));
        let app = build_test_app(repository, Arc::new(RecordingEventRecorder::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/orders/order-1")
                    .header("cookie", "access_token=token-other-user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_order_keeps_commercial_status_even_when_workflow_has_progressed() {
        let repository = Arc::new(InMemoryOrderRepository::default());
        repository.seed_order(make_order("order-1", "customer-1", OrderStatus::Placed));
        let app = build_test_app(repository, Arc::new(RecordingEventRecorder::default()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/orders/order-1")
                    .header("cookie", "access_token=token-customer-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["status"], "placed");
    }
}
