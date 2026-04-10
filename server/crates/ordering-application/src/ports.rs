use crate::{ApplicationError, OrderListItemReadModel, OrderReadModel, OrderingEvent};
use async_trait::async_trait;
use ordering_food_ordering_domain::{Order, OrderId};
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
    async fn get_by_id(&self, order_id: &str) -> Result<Option<OrderReadModel>, ApplicationError>;

    async fn list_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<OrderListItemReadModel>, ApplicationError>;
}

#[async_trait]
pub trait OrderingEventRecorder: Send + Sync {
    async fn record(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderingEvent,
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

    pub async fn get_by_id_for_customer(
        &self,
        order_id: &str,
        customer_id: &str,
    ) -> Result<OrderReadModel, ApplicationError> {
        let order = self
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("order was not found"))?;

        if order.customer_id != customer_id {
            return Err(ApplicationError::not_found("order was not found"));
        }

        Ok(order)
    }

    pub async fn list_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<OrderListItemReadModel>, ApplicationError> {
        self.repository.list_by_customer(customer_id).await
    }
}
