use crate::ApplicationError;
use async_trait::async_trait;
use ordering_food_fulfillment_domain::{FulfillmentOrder, FulfillmentOrderId};
pub use ordering_food_platform_kernel::Clock;
use ordering_food_shared_kernel::Timestamp;
use std::{any::Any, sync::Arc};

pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

pub trait IdGenerator: Send + Sync {
    fn next_fulfillment_order_id(&self) -> FulfillmentOrderId;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError>;
    async fn commit(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
    async fn rollback(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait WorkflowOrderRepository: Send + Sync {
    async fn find_by_ordering_order_id(
        &self,
        tx: &mut dyn TransactionContext,
        ordering_order_id: &str,
    ) -> Result<Option<FulfillmentOrder>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        order: &FulfillmentOrder,
    ) -> Result<(), ApplicationError>;

    async fn update(
        &self,
        tx: &mut dyn TransactionContext,
        order: &FulfillmentOrder,
    ) -> Result<(), ApplicationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowOrderReadModel {
    pub fulfillment_order_id: String,
    pub ordering_order_id: String,
    pub store_id: String,
    pub status: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[async_trait]
pub trait WorkflowOrderReadRepository: Send + Sync {
    async fn get_by_ordering_order_id(
        &self,
        ordering_order_id: &str,
    ) -> Result<Option<WorkflowOrderReadModel>, ApplicationError>;
}

#[derive(Clone)]
pub struct WorkflowOrderQueryService {
    repository: Arc<dyn WorkflowOrderReadRepository>,
}

impl WorkflowOrderQueryService {
    pub fn new(repository: Arc<dyn WorkflowOrderReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_by_ordering_order_id(
        &self,
        ordering_order_id: &str,
    ) -> Result<Option<WorkflowOrderReadModel>, ApplicationError> {
        self.repository
            .get_by_ordering_order_id(ordering_order_id)
            .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommercialOrderProjectionItemReadModel {
    pub line_number: i32,
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
    pub line_total_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommercialOrderProjectionReadModel {
    pub order_id: String,
    pub customer_id: String,
    pub store_id: String,
    pub status: String,
    pub subtotal_amount: i64,
    pub total_amount: i64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub items: Vec<CommercialOrderProjectionItemReadModel>,
}

#[async_trait]
pub trait CommercialOrderProjectionReadRepository: Send + Sync {
    async fn get_by_ordering_order_id(
        &self,
        ordering_order_id: &str,
    ) -> Result<Option<CommercialOrderProjectionReadModel>, ApplicationError>;
}

#[async_trait]
pub trait CommercialOrderProjectionStore: Send + Sync {
    async fn upsert(
        &self,
        tx: &mut dyn TransactionContext,
        projection: &CommercialOrderProjectionReadModel,
    ) -> Result<(), ApplicationError>;

    async fn update_status(
        &self,
        tx: &mut dyn TransactionContext,
        ordering_order_id: &str,
        status: &str,
        updated_at: Timestamp,
    ) -> Result<(), ApplicationError>;
}

#[derive(Clone)]
pub struct CommercialOrderProjectionQueryService {
    repository: Arc<dyn CommercialOrderProjectionReadRepository>,
}

impl CommercialOrderProjectionQueryService {
    pub fn new(repository: Arc<dyn CommercialOrderProjectionReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_by_ordering_order_id(
        &self,
        ordering_order_id: &str,
    ) -> Result<Option<CommercialOrderProjectionReadModel>, ApplicationError> {
        self.repository
            .get_by_ordering_order_id(ordering_order_id)
            .await
    }

    pub async fn ensure_workflow_transition_allowed(
        &self,
        ordering_order_id: &str,
    ) -> Result<CommercialOrderProjectionReadModel, ApplicationError> {
        let projection = self
            .get_by_ordering_order_id(ordering_order_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::not_found("commercial order projection was not found")
            })?;

        if projection.status != "placed" {
            return Err(ApplicationError::conflict(
                "commercial state forbids workflow transition",
            ));
        }

        Ok(projection)
    }
}
