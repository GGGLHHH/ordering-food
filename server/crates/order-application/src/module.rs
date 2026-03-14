use crate::{
    AcceptOrder, CancelOrderByCustomer, Clock, CompleteOrder, IdGenerator, MarkOrderReadyForPickup,
    OrderQueryService, OrderReadRepository, OrderRepository, PlaceOrderFromCart,
    RejectOrderByStore, StartPreparingOrder, TransactionManager,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct OrderModule {
    pub place_order_from_cart: Arc<PlaceOrderFromCart>,
    pub accept_order: Arc<AcceptOrder>,
    pub start_preparing_order: Arc<StartPreparingOrder>,
    pub mark_order_ready_for_pickup: Arc<MarkOrderReadyForPickup>,
    pub complete_order: Arc<CompleteOrder>,
    pub cancel_order_by_customer: Arc<CancelOrderByCustomer>,
    pub reject_order_by_store: Arc<RejectOrderByStore>,
    pub order_queries: Arc<OrderQueryService>,
}

impl OrderModule {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        order_read_repository: Arc<dyn OrderReadRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            place_order_from_cart: Arc::new(PlaceOrderFromCart::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
                id_generator,
            )),
            accept_order: Arc::new(AcceptOrder::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            start_preparing_order: Arc::new(StartPreparingOrder::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            mark_order_ready_for_pickup: Arc::new(MarkOrderReadyForPickup::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            complete_order: Arc::new(CompleteOrder::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            cancel_order_by_customer: Arc::new(CancelOrderByCustomer::new(
                order_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
            )),
            reject_order_by_store: Arc::new(RejectOrderByStore::new(
                order_repository,
                transaction_manager,
                clock,
            )),
            order_queries: Arc::new(OrderQueryService::new(order_read_repository)),
        }
    }
}
