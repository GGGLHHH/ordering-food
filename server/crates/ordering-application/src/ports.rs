use crate::{ApplicationError, OrderListItemReadModel, OrderReadModel};
use async_trait::async_trait;
use ordering_food_ordering_domain::{Order, OrderId};
pub use ordering_food_ordering_published::{
    OrderCancelledByCustomer, OrderCommercialStateChanged, OrderPlaced,
};
pub use ordering_food_platform_kernel::Clock;
use std::{any::Any, sync::Arc};

pub trait IdGenerator: Send + Sync {
    fn next_order_id(&self) -> OrderId;
}

pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError>;
    async fn commit(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
    async fn rollback(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        order_id: &OrderId,
    ) -> Result<Option<Order>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        order: &Order,
    ) -> Result<(), ApplicationError>;

    async fn update(
        &self,
        tx: &mut dyn TransactionContext,
        order: &Order,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait OrderReadRepository: Send + Sync {
    async fn get_by_id(
        &self,
        order_id: &str,
    ) -> Result<Option<OrderReadModel>, ApplicationError>;

    async fn list_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<OrderListItemReadModel>, ApplicationError>;
}

#[async_trait]
pub trait OrderingPublishedEventRecorder: Send + Sync {
    async fn record_order_placed(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderPlaced,
    ) -> Result<(), ApplicationError>;

    async fn record_order_commercial_state_changed(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderCommercialStateChanged,
    ) -> Result<(), ApplicationError>;

    async fn record_order_cancelled_by_customer(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderCancelledByCustomer,
    ) -> Result<(), ApplicationError>;
}

#[derive(Clone)]
pub struct OrderQueryService {
    repository: Arc<dyn OrderReadRepository>,
}

impl OrderQueryService {
    pub fn new(repository: Arc<dyn OrderReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_by_id(
        &self,
        order_id: &str,
    ) -> Result<Option<OrderReadModel>, ApplicationError> {
        self.repository.get_by_id(order_id).await
    }

    pub async fn list_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<OrderListItemReadModel>, ApplicationError> {
        self.repository.list_by_customer(customer_id).await
    }
}
