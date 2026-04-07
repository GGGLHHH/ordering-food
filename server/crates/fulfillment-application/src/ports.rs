use crate::ApplicationError;
use async_trait::async_trait;
use ordering_food_fulfillment_domain::{FulfillmentOrder, FulfillmentOrderId};
pub use ordering_food_ordering_published::{
    OrderCancelledByCustomer, OrderCommercialStateChanged, OrderPlaced, OrderPlacedItem,
};
pub use ordering_food_platform_kernel::Clock;
use ordering_food_shared_kernel::Timestamp;
use serde_json::Value;
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

// --- Outbox & Projection Checkpoint Ports ---

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxMessage {
    pub id: i64,
    pub producer_context: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: Value,
    pub occurred_at: Timestamp,
    pub available_at: Timestamp,
    pub error_count: i32,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionCheckpoint {
    pub projector_name: String,
    pub last_processed_id: i64,
    pub updated_at: Timestamp,
}

#[async_trait]
pub trait OutboxMessageReader: Send + Sync {
    async fn list_available(
        &self,
        producer_context: &str,
        after_id: i64,
        available_before: Timestamp,
        limit: i64,
    ) -> Result<Vec<OutboxMessage>, ApplicationError>;

    async fn record_failure(
        &self,
        message_id: i64,
        last_error: &str,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait ProjectionCheckpointStore: Send + Sync {
    async fn get(
        &self,
        projector_name: &str,
    ) -> Result<ProjectionCheckpoint, ApplicationError>;

    async fn save(
        &self,
        projector_name: &str,
        last_processed_id: i64,
        updated_at: Timestamp,
    ) -> Result<(), ApplicationError>;
}
