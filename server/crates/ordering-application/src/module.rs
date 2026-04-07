use crate::{
    CancelOrderByCustomer, Clock, IdGenerator, OrderQueryService, OrderReadRepository,
    OrderRepository, OrderingPublishedEventRecorder, PlaceOrderFromCart, TransactionManager,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct OrderingModule {
    place_order_from_cart: Arc<PlaceOrderFromCart>,
    cancel_order_by_customer: Arc<CancelOrderByCustomer>,
    order_queries: Arc<OrderQueryService>,
}

impl OrderingModule {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        order_read_repository: Arc<dyn OrderReadRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        event_recorder: Arc<dyn OrderingPublishedEventRecorder>,
    ) -> Self {
        Self {
            place_order_from_cart: Arc::new(PlaceOrderFromCart::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
                id_generator,
                event_recorder.clone(),
            )),
            cancel_order_by_customer: Arc::new(CancelOrderByCustomer::new(
                order_repository,
                transaction_manager,
                clock,
                event_recorder,
            )),
            order_queries: Arc::new(OrderQueryService::new(order_read_repository)),
        }
    }

    pub fn place_order_from_cart(&self) -> &Arc<PlaceOrderFromCart> {
        &self.place_order_from_cart
    }

    pub fn cancel_order_by_customer(&self) -> &Arc<CancelOrderByCustomer> {
        &self.cancel_order_by_customer
    }

    pub fn order_queries(&self) -> &Arc<OrderQueryService> {
        &self.order_queries
    }
}
