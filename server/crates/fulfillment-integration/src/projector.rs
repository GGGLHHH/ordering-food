use ordering_food_fulfillment_application::{
    ApplicationError, CommercialOrderCancelledByCustomer, CommercialOrderPlaced,
    CommercialOrderPlacedItem, CommercialOrderStateChanged, FulfillmentModule,
    OrderingCommercialEventHandler, WorkflowActionAuthorizer,
};
use ordering_food_fulfillment_infrastructure_sqlx::{
    SqlxOutboxMessageRecord, SqlxOutboxMessageRepository, SqlxProjectionCheckpointStore,
    build_fulfillment_module, build_ordering_commercial_event_handler_with_uuid_ids,
};
use ordering_food_ordering_published::{
    COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE, COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
    COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE, CommercialOrderCancelledByCustomerV1,
    CommercialOrderLineSnapshotV1, CommercialOrderPlacedV1, CommercialOrderStatusChangedV1,
};
use ordering_food_platform_kernel::Clock;
use ordering_food_shared_kernel::Timestamp;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;

pub const DEFAULT_ORDERING_EVENT_PROJECTOR_NAME: &str = "fulfillment.ordering-commercial";
const ORDERING_PRODUCER_CONTEXT: &str = "ordering";
const DEFAULT_BATCH_SIZE: i64 = 100;

#[derive(Clone)]
pub struct FulfillmentContextRuntime {
    module: Arc<FulfillmentModule>,
    ordering_event_projector: OrderingEventProjector,
}

impl FulfillmentContextRuntime {
    pub fn module(&self) -> &Arc<FulfillmentModule> {
        &self.module
    }

    pub fn ordering_event_projector(&self) -> &OrderingEventProjector {
        &self.ordering_event_projector
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderingEventProjectorRunResult {
    pub scanned_count: usize,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub last_processed_id: i64,
}

#[derive(Debug, Error)]
pub enum OrderingEventProjectorError {
    #[error(transparent)]
    Application(#[from] ApplicationError),
    #[error("failed to decode outbox event `{event_type}`")]
    Decode {
        event_type: String,
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct OutboxMessageEnvelope {
    id: i64,
    event_type: String,
    payload: serde_json::Value,
    occurred_at: Timestamp,
    available_at: Timestamp,
}

impl From<SqlxOutboxMessageRecord> for OutboxMessageEnvelope {
    fn from(message: SqlxOutboxMessageRecord) -> Self {
        Self {
            id: message.id,
            event_type: message.event_type,
            payload: message.payload,
            occurred_at: message.occurred_at,
            available_at: message.available_at,
        }
    }
}

enum DecodedOrderingEvent {
    Placed(CommercialOrderPlaced),
    CommercialStateChanged(CommercialOrderStateChanged),
    CancelledByCustomer(CommercialOrderCancelledByCustomer),
}

#[derive(Clone)]
pub struct OrderingEventProjector {
    projector_name: String,
    batch_size: i64,
    clock: Arc<dyn Clock>,
    outbox_messages: SqlxOutboxMessageRepository,
    checkpoints: SqlxProjectionCheckpointStore,
    event_handler: Arc<OrderingCommercialEventHandler>,
}

impl OrderingEventProjector {
    pub fn new(
        projector_name: impl Into<String>,
        batch_size: i64,
        clock: Arc<dyn Clock>,
        outbox_messages: SqlxOutboxMessageRepository,
        checkpoints: SqlxProjectionCheckpointStore,
        event_handler: Arc<OrderingCommercialEventHandler>,
    ) -> Self {
        Self {
            projector_name: projector_name.into(),
            batch_size,
            clock,
            outbox_messages,
            checkpoints,
            event_handler,
        }
    }

    pub async fn project_once(
        &self,
    ) -> Result<OrderingEventProjectorRunResult, OrderingEventProjectorError> {
        let checkpoint = self.checkpoints.get(&self.projector_name).await?;
        let messages = self
            .outbox_messages
            .list_available(
                ORDERING_PRODUCER_CONTEXT,
                checkpoint.last_processed_id,
                self.clock.now(),
                self.batch_size,
            )
            .await?;

        let mut result = OrderingEventProjectorRunResult {
            scanned_count: 0,
            applied_count: 0,
            skipped_count: 0,
            last_processed_id: checkpoint.last_processed_id,
        };

        for message in messages {
            let envelope = OutboxMessageEnvelope::from(message);
            let message_id = envelope.id;
            let event = match decode_ordering_event(&envelope) {
                Ok(event) => event,
                Err(error) => {
                    if let Err(mark_error) = self
                        .outbox_messages
                        .record_failure(message_id, &error.to_string())
                        .await
                    {
                        return Err(mark_error.into());
                    }

                    return Err(error);
                }
            };

            if let Some(event) = event {
                match event {
                    DecodedOrderingEvent::Placed(event) => {
                        self.event_handler.handle_order_placed(&event).await?;
                    }
                    DecodedOrderingEvent::CommercialStateChanged(event) => {
                        self.event_handler
                            .handle_order_commercial_state_changed(&event)
                            .await?;
                    }
                    DecodedOrderingEvent::CancelledByCustomer(event) => {
                        self.event_handler
                            .handle_order_cancelled_by_customer(&event)
                            .await?;
                    }
                }
                result.applied_count += 1;
            } else {
                result.skipped_count += 1;
            }

            result.scanned_count += 1;
            result.last_processed_id = message_id;
            self.checkpoints
                .save(&self.projector_name, message_id, self.clock.now())
                .await?;
        }

        Ok(result)
    }
}

pub fn build_ordering_event_projector(
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
) -> OrderingEventProjector {
    OrderingEventProjector::new(
        DEFAULT_ORDERING_EVENT_PROJECTOR_NAME,
        DEFAULT_BATCH_SIZE,
        clock,
        SqlxOutboxMessageRepository::new(pg_pool.clone()),
        SqlxProjectionCheckpointStore::new(pg_pool.clone()),
        build_ordering_commercial_event_handler_with_uuid_ids(pg_pool),
    )
}

pub fn build_fulfillment_context_runtime(
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
    workflow_action_authorizer: Arc<dyn WorkflowActionAuthorizer>,
) -> FulfillmentContextRuntime {
    FulfillmentContextRuntime {
        module: build_fulfillment_module(
            pg_pool.clone(),
            clock.clone(),
            workflow_action_authorizer,
        ),
        ordering_event_projector: build_ordering_event_projector(pg_pool, clock),
    }
}

fn decode_ordering_event(
    message: &OutboxMessageEnvelope,
) -> Result<Option<DecodedOrderingEvent>, OrderingEventProjectorError> {
    match message.event_type.as_str() {
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE => {
            serde_json::from_value::<CommercialOrderPlacedV1>(message.payload.clone())
                .map(map_order_placed)
                .map(DecodedOrderingEvent::Placed)
                .map(Some)
                .map_err(|source| OrderingEventProjectorError::Decode {
                    event_type: message.event_type.clone(),
                    source,
                })
        }
        COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE => {
            serde_json::from_value::<CommercialOrderStatusChangedV1>(message.payload.clone())
                .map(map_order_commercial_state_changed)
                .map(DecodedOrderingEvent::CommercialStateChanged)
                .map(Some)
                .map_err(|source| OrderingEventProjectorError::Decode {
                    event_type: message.event_type.clone(),
                    source,
                })
        }
        COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE => {
            serde_json::from_value::<CommercialOrderCancelledByCustomerV1>(message.payload.clone())
                .map(map_order_cancelled_by_customer)
                .map(DecodedOrderingEvent::CancelledByCustomer)
                .map(Some)
                .map_err(|source| OrderingEventProjectorError::Decode {
                    event_type: message.event_type.clone(),
                    source,
                })
        }
        _ => Ok(None),
    }
}

fn map_order_placed(event: CommercialOrderPlacedV1) -> CommercialOrderPlaced {
    CommercialOrderPlaced {
        order_id: event.order_id,
        customer_id: event.customer_id,
        store_id: event.store_id,
        status: "placed".to_string(),
        subtotal_amount: event.subtotal_amount,
        total_amount: event.total_amount,
        created_at: event.occurred_at,
        updated_at: event.occurred_at,
        items: event.items.into_iter().map(map_order_placed_item).collect(),
    }
}

fn map_order_placed_item(item: CommercialOrderLineSnapshotV1) -> CommercialOrderPlacedItem {
    CommercialOrderPlacedItem {
        line_number: item.line_number,
        catalog_item_id: item.catalog_item_id,
        name: item.name,
        unit_price_amount: item.unit_price_amount,
        quantity: item.quantity,
        line_total_amount: item.line_total_amount,
    }
}

fn map_order_commercial_state_changed(
    event: CommercialOrderStatusChangedV1,
) -> CommercialOrderStateChanged {
    CommercialOrderStateChanged {
        order_id: event.order_id,
        customer_id: event.customer_id,
        store_id: event.store_id,
        previous_status: event.previous_status,
        current_status: event.current_status,
        occurred_at: event.occurred_at,
    }
}

fn map_order_cancelled_by_customer(
    event: CommercialOrderCancelledByCustomerV1,
) -> CommercialOrderCancelledByCustomer {
    CommercialOrderCancelledByCustomer {
        order_id: event.order_id,
        customer_id: event.customer_id,
        store_id: event.store_id,
        occurred_at: event.occurred_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn order_placed_payload_decodes_into_local_semantic_event() {
        let message = OutboxMessageEnvelope {
            id: 1,
            event_type: COMMERCIAL_ORDER_PLACED_EVENT_TYPE.to_string(),
            payload: serde_json::json!(CommercialOrderPlacedV1 {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                subtotal_amount: 1800,
                total_amount: 1800,
                occurred_at: datetime!(2026-03-15 10:00 UTC),
                items: vec![CommercialOrderLineSnapshotV1 {
                    line_number: 1,
                    catalog_item_id: "item-1".to_string(),
                    name: "Noodles".to_string(),
                    unit_price_amount: 1800,
                    quantity: 1,
                    line_total_amount: 1800,
                }],
            }),
            occurred_at: datetime!(2026-03-15 10:00 UTC),
            available_at: datetime!(2026-03-15 10:00 UTC),
        };

        let event = decode_ordering_event(&message).unwrap();

        assert!(matches!(
            event,
            Some(DecodedOrderingEvent::Placed(CommercialOrderPlaced {
                order_id,
                customer_id,
                store_id,
                status,
                subtotal_amount,
                total_amount,
                created_at,
                updated_at,
                ..
            })) if order_id == "order-1"
                && customer_id == "customer-1"
                && store_id == "store-1"
                && status == "placed"
                && subtotal_amount == 1800
                && total_amount == 1800
                && created_at == datetime!(2026-03-15 10:00 UTC)
                && updated_at == datetime!(2026-03-15 10:00 UTC)
        ));
    }

    #[test]
    fn status_changed_payload_decodes_into_local_semantic_event() {
        let message = OutboxMessageEnvelope {
            id: 2,
            event_type: COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE.to_string(),
            payload: serde_json::json!(CommercialOrderStatusChangedV1 {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                previous_status: "placed".to_string(),
                current_status: "accepted".to_string(),
                occurred_at: datetime!(2026-03-15 10:05 UTC),
            }),
            occurred_at: datetime!(2026-03-15 10:05 UTC),
            available_at: datetime!(2026-03-15 10:05 UTC),
        };

        let event = decode_ordering_event(&message).unwrap();

        assert!(matches!(
            event,
            Some(DecodedOrderingEvent::CommercialStateChanged(CommercialOrderStateChanged {
                order_id,
                customer_id,
                store_id,
                previous_status,
                current_status,
                occurred_at,
            })) if order_id == "order-1"
                && customer_id == "customer-1"
                && store_id == "store-1"
                && previous_status == "placed"
                && current_status == "accepted"
                && occurred_at == datetime!(2026-03-15 10:05 UTC)
        ));
    }

    #[test]
    fn cancelled_payload_decodes_into_local_semantic_event() {
        let message = OutboxMessageEnvelope {
            id: 3,
            event_type: COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE.to_string(),
            payload: serde_json::json!(CommercialOrderCancelledByCustomerV1 {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                occurred_at: datetime!(2026-03-15 10:10 UTC),
            }),
            occurred_at: datetime!(2026-03-15 10:10 UTC),
            available_at: datetime!(2026-03-15 10:10 UTC),
        };

        let event = decode_ordering_event(&message).unwrap();

        assert!(matches!(
            event,
            Some(DecodedOrderingEvent::CancelledByCustomer(
                CommercialOrderCancelledByCustomer {
                    order_id,
                    customer_id,
                    store_id,
                    occurred_at,
                }
            )) if order_id == "order-1"
                && customer_id == "customer-1"
                && store_id == "store-1"
                && occurred_at == datetime!(2026-03-15 10:10 UTC)
        ));
    }

    #[test]
    fn unknown_ordering_event_type_is_skipped() {
        let message = OutboxMessageEnvelope {
            id: 1,
            event_type: "ordering.order_unknown".to_string(),
            payload: serde_json::json!({}),
            occurred_at: datetime!(2026-03-15 10:00 UTC),
            available_at: datetime!(2026-03-15 10:00 UTC),
        };

        let event = decode_ordering_event(&message).unwrap();

        assert!(event.is_none());
    }
}
