use crate::{
    app::AppState,
    error::{AppError, ErrorEnvelope},
    http::{self, ApiJson, ApiPath, AuthenticatedUser, RequestContext},
};
use axum::{
    Extension, Json, Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use ordering_food_order_application::{
    AcceptOrderInput, ApplicationError, CancelOrderByCustomerInput, CompleteOrderInput,
    MarkOrderReadyForPickupInput, OrderModule, PlaceOrderFromCartInput,
    PlaceOrderItemInput as ApplicationPlaceOrderItemInput, RejectOrderByStoreInput,
    StartPreparingOrderInput,
};
use ordering_food_order_domain::OrderId;
use ordering_food_shared_kernel::Identifier;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use ts_rs::TS;
use utoipa::{IntoParams, OpenApi, ToSchema};

pub(crate) const ORDER_ROUTE_PREFIX: &str = "/api/orders";
pub(crate) const ORDERS_PATH: &str = "/api/orders";
pub(crate) const ORDER_PATH: &str = "/api/orders/{order_id}";
pub(crate) const ORDER_CANCEL_PATH: &str = "/api/orders/{order_id}/cancel";
pub(crate) const ORDER_ACCEPT_PATH: &str = "/api/orders/{order_id}/accept";
pub(crate) const ORDER_START_PREPARING_PATH: &str = "/api/orders/{order_id}/start-preparing";
pub(crate) const ORDER_READY_PATH: &str = "/api/orders/{order_id}/ready";
pub(crate) const ORDER_COMPLETE_PATH: &str = "/api/orders/{order_id}/complete";
pub(crate) const ORDER_REJECT_PATH: &str = "/api/orders/{order_id}/reject";

const ROOT_ROUTE_PATH: &str = "/";
const DETAIL_ROUTE_PATH: &str = "/{order_id}";
const CANCEL_ROUTE_PATH: &str = "/{order_id}/cancel";
const ACCEPT_ROUTE_PATH: &str = "/{order_id}/accept";
const START_PREPARING_ROUTE_PATH: &str = "/{order_id}/start-preparing";
const READY_ROUTE_PATH: &str = "/{order_id}/ready";
const COMPLETE_ROUTE_PATH: &str = "/{order_id}/complete";
const REJECT_ROUTE_PATH: &str = "/{order_id}/reject";

pub fn router(module: Arc<OrderModule>) -> Router<AppState> {
    Router::new()
        .route(ROOT_ROUTE_PATH, post(place_order))
        .route(DETAIL_ROUTE_PATH, get(get_order))
        .route(CANCEL_ROUTE_PATH, post(cancel_order))
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
        place_order,
        get_order,
        cancel_order,
        accept_order,
        start_preparing_order,
        mark_order_ready,
        complete_order,
        reject_order,
    ),
    components(
        schemas(
            ErrorEnvelope,
            PlaceOrderItemRequest,
            PlaceOrderRequest,
            OrderPath,
            OrderItemResponse,
            OrderResponse,
        )
    ),
    tags(
        (name = "orders", description = "Order management endpoints")
    )
)]
pub struct OrderApiDoc;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
pub struct PlaceOrderItemRequest {
    pub menu_item_id: String,
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
    pub menu_item_id: String,
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
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    user: AuthenticatedUser,
    ApiJson(payload): ApiJson<PlaceOrderRequest>,
) -> Result<Json<OrderResponse>, AppError> {
    let order = module
        .place_order_from_cart
        .execute(PlaceOrderFromCartInput {
            customer_id: user.user_id.clone(),
            store_id: payload.store_id,
            items: payload
                .items
                .into_iter()
                .map(|item| ApplicationPlaceOrderItemInput {
                    menu_item_id: item.menu_item_id,
                    name: item.name,
                    unit_price_amount: item.unit_price_amount,
                    quantity: item.quantity,
                })
                .collect(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response =
        load_order_response(&module, order.id().as_str(), None, context.request_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = ORDER_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Get order detail", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn get_order(
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    let response = load_order_response(
        &module,
        &path.order_id,
        Some(&user.user_id),
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
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .cancel_order_by_customer
        .execute(CancelOrderByCustomerInput {
            order_id: path.order_id.clone(),
            customer_id: user.user_id.clone(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response = load_order_response(
        &module,
        &path.order_id,
        Some(&user.user_id),
        context.request_id,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_ACCEPT_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Accept order", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be accepted", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn accept_order(
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    _user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .accept_order
        .execute(AcceptOrderInput {
            order_id: path.order_id.clone(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response = load_order_response(&module, &path.order_id, None, context.request_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_START_PREPARING_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Start preparing order", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot start preparing", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn start_preparing_order(
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    _user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .start_preparing_order
        .execute(StartPreparingOrderInput {
            order_id: path.order_id.clone(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response = load_order_response(&module, &path.order_id, None, context.request_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_READY_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Mark order ready for pickup", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be marked ready", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn mark_order_ready(
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    _user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .mark_order_ready_for_pickup
        .execute(MarkOrderReadyForPickupInput {
            order_id: path.order_id.clone(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response = load_order_response(&module, &path.order_id, None, context.request_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_COMPLETE_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Complete order", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be completed", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn complete_order(
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    _user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .complete_order
        .execute(CompleteOrderInput {
            order_id: path.order_id.clone(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response = load_order_response(&module, &path.order_id, None, context.request_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = ORDER_REJECT_PATH,
    tag = "orders",
    params(OrderPath),
    responses(
        (status = 200, description = "Reject order", body = OrderResponse),
        (status = 401, description = "Not authenticated", body = ErrorEnvelope),
        (status = 404, description = "Order was not found", body = ErrorEnvelope),
        (status = 409, description = "Order cannot be rejected", body = ErrorEnvelope),
        (status = 422, description = "Path validation failed", body = ErrorEnvelope),
        (status = 500, description = "Internal server error", body = ErrorEnvelope)
    )
)]
pub async fn reject_order(
    Extension(module): Extension<Arc<OrderModule>>,
    context: RequestContext,
    _user: AuthenticatedUser,
    ApiPath(path): ApiPath<OrderPath>,
) -> Result<Json<OrderResponse>, AppError> {
    module
        .reject_order_by_store
        .execute(RejectOrderByStoreInput {
            order_id: path.order_id.clone(),
        })
        .await
        .map_err(|error| map_order_error(error, context.request_id.clone()))?;

    let response = load_order_response(&module, &path.order_id, None, context.request_id).await?;
    Ok(Json(response))
}

async fn load_order_response(
    module: &OrderModule,
    order_id: &str,
    customer_id: Option<&str>,
    request_id: Option<String>,
) -> Result<OrderResponse, AppError> {
    let order = module
        .order_queries
        .get_by_id(&OrderId::new(order_id.to_string()))
        .await
        .map_err(|error| map_order_error(error, request_id.clone()))?
        .ok_or_else(|| {
            AppError::not_found("order was not found").with_request_id(request_id.clone())
        })?;

    if customer_id.is_some_and(|customer_id| customer_id != order.customer_id) {
        return Err(AppError::not_found("order was not found").with_request_id(request_id));
    }

    map_order_response(order).map_err(|error| error.with_request_id(request_id))
}

fn map_order_response(
    order: ordering_food_order_application::OrderReadModel,
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
                menu_item_id: item.menu_item_id,
                name: item.name,
                unit_price_amount: item.unit_price_amount,
                quantity: item.quantity,
                line_total_amount: item.line_total_amount,
            })
            .collect(),
    })
}

fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, AppError> {
    timestamp
        .format(&Rfc3339)
        .map_err(|error| AppError::internal_with_source("internal server error", error))
}

fn map_order_error(error: ApplicationError, request_id: Option<String>) -> AppError {
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
        ApplicationError::Unexpected { .. } => {
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
        Extension, Router,
        body::{Body, to_bytes},
        http::{HeaderName, Request, StatusCode},
        response::Response,
    };
    use ordering_food_identity_application::{AccessTokenClaims, TokenPair, TokenService};
    use ordering_food_order_application::{
        Clock, IdGenerator, OrderItemReadModel, OrderModule, OrderReadModel, OrderReadRepository,
        OrderRepository, TransactionContext, TransactionManager,
    };
    use ordering_food_order_domain::{
        CustomerId, MenuItemId, Order, OrderId, OrderStatus, PlaceOrderItemInput, StoreId,
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
        ) -> Result<Option<OrderReadModel>, ApplicationError> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .orders
                .get(order_id.as_str())
                .map(|order| OrderReadModel {
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
                            menu_item_id: item.menu_item_id().as_str().to_string(),
                            name: item.name().to_string(),
                            unit_price_amount: item.unit_price_amount(),
                            quantity: item.quantity(),
                            line_total_amount: item.line_total_amount(),
                        })
                        .collect(),
                }))
        }
    }

    #[derive(Default)]
    struct FakeTokenService;

    #[async_trait]
    impl TokenService for FakeTokenService {
        fn generate_token_pair(
            &self,
            user_id: &str,
        ) -> Result<TokenPair, ordering_food_identity_application::ApplicationError> {
            Ok(TokenPair {
                access_token: format!("token-{user_id}"),
                access_token_expires_in: 900,
                refresh_token: "refresh".to_string(),
                refresh_token_expires_in: 604800,
            })
        }

        fn verify_access_token(
            &self,
            token: &str,
        ) -> Result<AccessTokenClaims, ordering_food_identity_application::ApplicationError>
        {
            let user_id = token
                .strip_prefix("token-")
                .ok_or_else(|| {
                    ordering_food_identity_application::ApplicationError::unauthorized(
                        "invalid or expired access token",
                    )
                })?
                .to_string();

            Ok(AccessTokenClaims { user_id, exp: 900 })
        }
    }

    struct StubReadiness;

    #[async_trait]
    impl ReadinessProbe for StubReadiness {
        async fn check(&self) -> Result<DependencyChecks, AppError> {
            Ok(DependencyChecks::ok("ok", "ok"))
        }
    }

    fn build_test_app(repository: Arc<InMemoryOrderRepository>) -> Router {
        let module = Arc::new(OrderModule::new(
            repository.clone(),
            repository,
            Arc::new(FakeTransactionManager),
            Arc::new(FakeClock {
                now: datetime!(2026-03-15 10:00 UTC),
            }),
            Arc::new(FakeIdGenerator),
        ));
        let request_id_header = HeaderName::from_static("x-request-id");
        let token_service: Arc<dyn TokenService> = Arc::new(FakeTokenService);

        Router::new()
            .nest(
                ORDER_ROUTE_PREFIX,
                router(module).layer(Extension(token_service)),
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
                menu_item_id: MenuItemId::new("item-1"),
                name: "Fried Rice".to_string(),
                unit_price_amount: 3200,
                quantity: 1,
            }],
            datetime!(2026-03-15 09:00 UTC),
        )
        .unwrap();

        match status {
            OrderStatus::PendingAcceptance => {}
            OrderStatus::Accepted => {
                order.accept(datetime!(2026-03-15 09:01 UTC)).unwrap();
            }
            OrderStatus::Preparing => {
                order.accept(datetime!(2026-03-15 09:01 UTC)).unwrap();
                order
                    .start_preparing(datetime!(2026-03-15 09:02 UTC))
                    .unwrap();
            }
            _ => unreachable!("test helper only supports pending/accepted/preparing"),
        }

        order
    }

    #[tokio::test]
    async fn place_order_returns_snapshot_for_authenticated_user() {
        let app = build_test_app(Arc::new(InMemoryOrderRepository::default()));

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
                                "menu_item_id": "item-1",
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
        assert_eq!(body["customer_id"], "customer-1");
        assert_eq!(body["status"], "pending_acceptance");
        assert_eq!(body["items"][0]["line_total_amount"], 6400);
    }

    #[tokio::test]
    async fn get_order_hides_other_users_order() {
        let repository = Arc::new(InMemoryOrderRepository::default());
        repository.seed_order(make_order(
            "order-1",
            "customer-1",
            OrderStatus::PendingAcceptance,
        ));
        let app = build_test_app(repository);

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
    async fn cancel_order_returns_conflict_after_preparing() {
        let repository = Arc::new(InMemoryOrderRepository::default());
        repository.seed_order(make_order("order-1", "customer-1", OrderStatus::Preparing));
        let app = build_test_app(repository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/orders/order-1/cancel")
                    .header("cookie", "access_token=token-customer-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn merchant_actions_advance_order_status() {
        let repository = Arc::new(InMemoryOrderRepository::default());
        repository.seed_order(make_order(
            "order-1",
            "customer-1",
            OrderStatus::PendingAcceptance,
        ));
        let app = build_test_app(repository.clone());

        for (path, expected_status) in [
            ("/api/orders/order-1/accept", "accepted"),
            ("/api/orders/order-1/start-preparing", "preparing"),
            ("/api/orders/order-1/ready", "ready_for_pickup"),
            ("/api/orders/order-1/complete", "completed"),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(path)
                        .header("cookie", "access_token=token-merchant-1")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            let body = response_json(response).await;
            assert_eq!(body["status"], expected_status);
        }
    }
}
