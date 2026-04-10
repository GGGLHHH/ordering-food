use async_trait::async_trait;
use ordering_food_ordering_application::{
    ApplicationError, LocalCommercialOrderLineSnapshot, OrderingEvent, OrderingEventRecorder,
    TransactionContext,
};
use ordering_food_ordering_infrastructure_sqlx::{
    OutboxMessageWriteRequest, SqlxOutboxMessageAppender,
};
use ordering_food_ordering_published::{
    CommercialOrderCancelledByCustomerV1, CommercialOrderLineSnapshotV1,
    CommercialOrderPlacedV1, CommercialOrderStatusChangedV1,
    COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE,
    COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
    COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE,
};
use ordering_food_shared_kernel::Timestamp;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

const ORDERING_PRODUCER_CONTEXT: &str = "ordering";

#[derive(Debug, Clone, PartialEq)]
pub struct PublishedOutboxEvent {
    pub event_type: &'static str,
    pub aggregate_id: String,
    pub occurred_at: Timestamp,
    pub payload: Value,
}

pub fn map_ordering_event_to_published(
    event: &OrderingEvent,
) -> Result<PublishedOutboxEvent, ApplicationError> {
    match event {
        OrderingEvent::CommercialOrderPlaced(local) => {
            let payload = CommercialOrderPlacedV1 {
                order_id: local.order_id.clone(),
                customer_id: local.customer_id.clone(),
                store_id: local.store_id.clone(),
                subtotal_amount: local.subtotal_amount,
                total_amount: local.total_amount,
                occurred_at: local.occurred_at,
                items: local
                    .items
                    .iter()
                    .map(map_line_snapshot)
                    .collect(),
            };

            published_outbox_event(
                COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
                &local.order_id,
                local.occurred_at,
                &payload,
            )
        }
        OrderingEvent::CommercialOrderStatusChanged(local) => {
            let payload = CommercialOrderStatusChangedV1 {
                order_id: local.order_id.clone(),
                customer_id: local.customer_id.clone(),
                store_id: local.store_id.clone(),
                previous_status: local.previous_status.clone(),
                current_status: local.current_status.clone(),
                occurred_at: local.occurred_at,
            };

            published_outbox_event(
                COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE,
                &local.order_id,
                local.occurred_at,
                &payload,
            )
        }
        OrderingEvent::CommercialOrderCancelledByCustomer(local) => {
            let payload = CommercialOrderCancelledByCustomerV1 {
                order_id: local.order_id.clone(),
                customer_id: local.customer_id.clone(),
                store_id: local.store_id.clone(),
                occurred_at: local.occurred_at,
            };

            published_outbox_event(
                COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE,
                &local.order_id,
                local.occurred_at,
                &payload,
            )
        }
    }
}

pub struct AdapterBackedOrderingEventRecorder {
    outbox_message_appender: Arc<SqlxOutboxMessageAppender>,
}

impl AdapterBackedOrderingEventRecorder {
    pub fn new(outbox_message_appender: Arc<SqlxOutboxMessageAppender>) -> Self {
        Self {
            outbox_message_appender,
        }
    }
}

#[async_trait]
impl OrderingEventRecorder for AdapterBackedOrderingEventRecorder {
    async fn record(
        &self,
        tx: &mut dyn TransactionContext,
        event: &OrderingEvent,
    ) -> Result<(), ApplicationError> {
        let published = map_ordering_event_to_published(event)?;
        self.outbox_message_appender
            .append(
                tx,
                OutboxMessageWriteRequest {
                    producer_context: ORDERING_PRODUCER_CONTEXT.to_string(),
                    event_type: published.event_type.to_string(),
                    aggregate_id: published.aggregate_id,
                    payload: published.payload,
                    occurred_at: published.occurred_at,
                },
            )
            .await
    }
}

fn map_line_snapshot(local: &LocalCommercialOrderLineSnapshot) -> CommercialOrderLineSnapshotV1 {
    CommercialOrderLineSnapshotV1 {
        line_number: local.line_number,
        catalog_item_id: local.catalog_item_id.clone(),
        name: local.name.clone(),
        unit_price_amount: local.unit_price_amount,
        quantity: local.quantity,
        line_total_amount: local.line_total_amount,
    }
}

fn published_outbox_event<T: Serialize>(
    event_type: &'static str,
    aggregate_id: &str,
    occurred_at: Timestamp,
    payload: &T,
) -> Result<PublishedOutboxEvent, ApplicationError> {
    Ok(PublishedOutboxEvent {
        event_type,
        aggregate_id: aggregate_id.to_string(),
        occurred_at,
        payload: serde_json::to_value(payload).map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to serialize ordering published event payload",
                error,
            )
        })?,
    })
}
