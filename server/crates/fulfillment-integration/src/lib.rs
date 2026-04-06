use ordering_food_fulfillment_application::{
    ApplicationError, FulfillmentModule, OrderCancelledByCustomer, OrderCommercialStateChanged,
    OrderPlaced, OrderingCommercialEventHandler, OutboxMessage, OutboxMessageReader,
    ProjectionCheckpointStore,
};
use ordering_food_fulfillment_infrastructure_sqlx::{
    SqlxOutboxMessageRepository, SqlxProjectionCheckpointStore, build_fulfillment_module,
    build_ordering_commercial_event_handler_with_uuid_ids,
};
use ordering_food_platform_kernel::Clock;
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

#[derive(Clone)]
pub struct OrderingEventProjector {
    projector_name: String,
    batch_size: i64,
    clock: Arc<dyn Clock>,
    outbox_messages: Arc<dyn OutboxMessageReader>,
    checkpoints: Arc<dyn ProjectionCheckpointStore>,
    event_handler: Arc<OrderingCommercialEventHandler>,
}

impl OrderingEventProjector {
    pub fn new(
        projector_name: impl Into<String>,
        batch_size: i64,
        clock: Arc<dyn Clock>,
        outbox_messages: Arc<dyn OutboxMessageReader>,
        checkpoints: Arc<dyn ProjectionCheckpointStore>,
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
            let event = match decode_ordering_event(&message) {
                Ok(event) => event,
                Err(error) => {
                    if let Err(mark_error) = self
                        .outbox_messages
                        .record_failure(message.id, &error.to_string())
                        .await
                    {
                        return Err(mark_error.into());
                    }

                    return Err(error);
                }
            };

            if let Some(event) = event {
                match event {
                    OrderingPublishedEvent::Placed(event) => {
                        self.event_handler.handle_order_placed(&event).await?;
                    }
                    OrderingPublishedEvent::CommercialStateChanged(event) => {
                        self.event_handler
                            .handle_order_commercial_state_changed(&event)
                            .await?;
                    }
                    OrderingPublishedEvent::CancelledByCustomer(event) => {
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
            result.last_processed_id = message.id;
            self.checkpoints
                .save(&self.projector_name, message.id, self.clock.now())
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
        Arc::new(SqlxOutboxMessageRepository::new(pg_pool.clone())),
        Arc::new(SqlxProjectionCheckpointStore::new(pg_pool.clone())),
        build_ordering_commercial_event_handler_with_uuid_ids(pg_pool),
    )
}

pub fn build_fulfillment_context_runtime(
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
) -> FulfillmentContextRuntime {
    FulfillmentContextRuntime {
        module: build_fulfillment_module(pg_pool.clone(), clock.clone()),
        ordering_event_projector: build_ordering_event_projector(pg_pool, clock),
    }
}

enum OrderingPublishedEvent {
    Placed(OrderPlaced),
    CommercialStateChanged(OrderCommercialStateChanged),
    CancelledByCustomer(OrderCancelledByCustomer),
}

fn decode_ordering_event(
    message: &OutboxMessage,
) -> Result<Option<OrderingPublishedEvent>, OrderingEventProjectorError> {
    match message.event_type.as_str() {
        "ordering.order_placed" => serde_json::from_value::<OrderPlaced>(message.payload.clone())
            .map(OrderingPublishedEvent::Placed)
            .map(Some)
            .map_err(|source| OrderingEventProjectorError::Decode {
                event_type: message.event_type.clone(),
                source,
            }),
        "ordering.order_commercial_state_changed" => {
            serde_json::from_value::<OrderCommercialStateChanged>(message.payload.clone())
                .map(OrderingPublishedEvent::CommercialStateChanged)
                .map(Some)
                .map_err(|source| OrderingEventProjectorError::Decode {
                    event_type: message.event_type.clone(),
                    source,
                })
        }
        "ordering.order_cancelled_by_customer" => {
            serde_json::from_value::<OrderCancelledByCustomer>(message.payload.clone())
                .map(OrderingPublishedEvent::CancelledByCustomer)
                .map(Some)
                .map_err(|source| OrderingEventProjectorError::Decode {
                    event_type: message.event_type.clone(),
                    source,
                })
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordering_food_ordering_published::{OrderPlaced, OrderPlacedItem};
    use time::macros::datetime;

    #[test]
    fn order_placed_payload_decodes_from_outbox_message() {
        let message = OutboxMessage {
            id: 1,
            producer_context: "ordering".to_string(),
            event_type: "ordering.order_placed".to_string(),
            aggregate_id: "order-1".to_string(),
            payload: serde_json::json!(OrderPlaced {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                status: "placed".to_string(),
                subtotal_amount: 1800,
                total_amount: 1800,
                created_at: datetime!(2026-03-15 10:00 UTC),
                updated_at: datetime!(2026-03-15 10:00 UTC),
                items: vec![OrderPlacedItem {
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
            error_count: 0,
            last_error: None,
            created_at: datetime!(2026-03-15 10:00 UTC),
        };

        let event = decode_ordering_event(&message).unwrap();

        assert!(matches!(event, Some(OrderingPublishedEvent::Placed(_))));
    }

    #[test]
    fn unknown_ordering_event_type_is_skipped() {
        let message = OutboxMessage {
            id: 1,
            producer_context: "ordering".to_string(),
            event_type: "ordering.order_unknown".to_string(),
            aggregate_id: "order-1".to_string(),
            payload: serde_json::json!({}),
            occurred_at: datetime!(2026-03-15 10:00 UTC),
            available_at: datetime!(2026-03-15 10:00 UTC),
            error_count: 0,
            last_error: None,
            created_at: datetime!(2026-03-15 10:00 UTC),
        };

        let event = decode_ordering_event(&message).unwrap();

        assert!(event.is_none());
    }
}
