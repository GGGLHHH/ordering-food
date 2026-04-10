use crate::{
    ApplicationError, Clock, IdGenerator, LocalCommercialOrderLineSnapshot,
    LocalCommercialOrderPlaced, OrderRepository, OrderingEvent, OrderingEventRecorder,
    TransactionManager,
};
use ordering_food_ordering_domain::{
    CatalogItemId, CustomerId, Order, PlaceOrderItemInput as DomainPlaceOrderItemInput, StoreId,
};
use ordering_food_shared_kernel::Identifier;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceOrderItemInput {
    pub catalog_item_id: String,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceOrderFromCartInput {
    pub customer_id: String,
    pub store_id: String,
    pub items: Vec<PlaceOrderItemInput>,
}

pub struct PlaceOrderFromCart {
    order_repository: Arc<dyn OrderRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    event_recorder: Arc<dyn OrderingEventRecorder>,
}

impl PlaceOrderFromCart {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        event_recorder: Arc<dyn OrderingEventRecorder>,
    ) -> Self {
        Self {
            order_repository,
            transaction_manager,
            clock,
            id_generator,
            event_recorder,
        }
    }

    pub async fn execute(
        &self,
        input: PlaceOrderFromCartInput,
    ) -> Result<String, ApplicationError> {
        let order = Order::place(
            self.id_generator.next_order_id(),
            CustomerId::new(input.customer_id),
            StoreId::new(input.store_id),
            input
                .items
                .into_iter()
                .map(|item| DomainPlaceOrderItemInput {
                    catalog_item_id: CatalogItemId::new(item.catalog_item_id),
                    name: item.name,
                    unit_price_amount: item.unit_price_amount,
                    quantity: item.quantity,
                })
                .collect(),
            self.clock.now(),
        )?;

        let mut tx = self.transaction_manager.begin().await?;
        if let Err(error) = self.order_repository.insert(tx.as_mut(), &order).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let event = OrderingEvent::CommercialOrderPlaced(LocalCommercialOrderPlaced {
            order_id: order.id().as_str().to_string(),
            customer_id: order.customer_id().as_str().to_string(),
            store_id: order.store_id().as_str().to_string(),
            subtotal_amount: order.subtotal_amount(),
            total_amount: order.total_amount(),
            occurred_at: order.updated_at(),
            items: order
                .items()
                .iter()
                .map(|item| LocalCommercialOrderLineSnapshot {
                    line_number: item.line_number(),
                    catalog_item_id: item.catalog_item_id().as_str().to_string(),
                    name: item.name().to_string(),
                    unit_price_amount: item.unit_price_amount(),
                    quantity: item.quantity(),
                    line_total_amount: item.line_total_amount(),
                })
                .collect(),
        });

        if let Err(error) = self.event_recorder.record(tx.as_mut(), &event).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(order.id().as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApplicationError, OrderRepository, OrderingEvent, OrderingEventRecorder, TransactionContext,
    };
    use async_trait::async_trait;
    use ordering_food_ordering_domain::{Order, OrderId};
    use ordering_food_shared_kernel::Timestamp;
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };
    use time::macros::datetime;

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
    struct RecordingTransactionManager {
        committed: Mutex<u32>,
        rolled_back: Mutex<u32>,
    }

    #[async_trait]
    impl crate::TransactionManager for RecordingTransactionManager {
        async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
            Ok(Box::new(FakeTransactionContext))
        }

        async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            *self.committed.lock().unwrap() += 1;
            Ok(())
        }

        async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
            *self.rolled_back.lock().unwrap() += 1;
            Ok(())
        }
    }

    struct FixedClock {
        now: Timestamp,
    }

    impl crate::Clock for FixedClock {
        fn now(&self) -> Timestamp {
            self.now
        }
    }

    struct FixedIdGenerator;

    impl crate::IdGenerator for FixedIdGenerator {
        fn next_order_id(&self) -> OrderId {
            OrderId::new("generated-order")
        }
    }

    #[derive(Default)]
    struct RecordingEventRecorder {
        events: Mutex<Vec<OrderingEvent>>,
        fail_record: bool,
    }

    #[async_trait]
    impl OrderingEventRecorder for RecordingEventRecorder {
        async fn record(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &OrderingEvent,
        ) -> Result<(), ApplicationError> {
            if self.fail_record {
                return Err(ApplicationError::unexpected("record failed"));
            }
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeOrderRepository {
        inserted: Mutex<Vec<Order>>,
        fail_insert: bool,
    }

    #[async_trait]
    impl OrderRepository for FakeOrderRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _order_id: &OrderId,
        ) -> Result<Option<Order>, ApplicationError> {
            Ok(None)
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            order: &Order,
        ) -> Result<(), ApplicationError> {
            if self.fail_insert {
                return Err(ApplicationError::unexpected("insert failed"));
            }
            self.inserted.lock().unwrap().push(order.clone());
            Ok(())
        }

        async fn update(
            &self,
            _tx: &mut dyn TransactionContext,
            _order: &Order,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn place_order_commits_on_success() {
        let repository = Arc::new(FakeOrderRepository::default());
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = PlaceOrderFromCart::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:00 UTC),
            }),
            Arc::new(FixedIdGenerator),
            event_recorder.clone(),
        );

        let order = use_case
            .execute(PlaceOrderFromCartInput {
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                items: vec![PlaceOrderItemInput {
                    catalog_item_id: "item-1".to_string(),
                    name: "Fried Rice".to_string(),
                    unit_price_amount: 3200,
                    quantity: 2,
                }],
            })
            .await
            .unwrap();

        assert_eq!(order, "generated-order");
        assert_eq!(repository.inserted.lock().unwrap().len(), 1);
        assert_eq!(*transactions.committed.lock().unwrap(), 1);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 0);
        assert_eq!(event_recorder.events.lock().unwrap().len(), 1);
        let first_event = event_recorder.events.lock().unwrap()[0].clone();
        let placed = match first_event {
            OrderingEvent::CommercialOrderPlaced(placed) => placed,
            _ => panic!("expected CommercialOrderPlaced event"),
        };
        assert_eq!(placed.order_id, "generated-order");
        assert_eq!(placed.customer_id, "customer-1".to_string());
        assert_eq!(placed.store_id, "store-1".to_string());
        assert_eq!(placed.subtotal_amount, 6400);
        assert_eq!(placed.total_amount, 6400);
        assert_eq!(placed.occurred_at, datetime!(2026-03-15 10:00 UTC));
        assert_eq!(placed.items.len(), 1);
        assert_eq!(placed.items[0].line_number, 1);
        assert_eq!(placed.items[0].catalog_item_id, "item-1");
        assert_eq!(placed.items[0].name, "Fried Rice");
        assert_eq!(placed.items[0].unit_price_amount, 3200);
        assert_eq!(placed.items[0].quantity, 2);
        assert_eq!(placed.items[0].line_total_amount, 6400);
    }

    #[tokio::test]
    async fn place_order_rolls_back_when_insert_fails() {
        let repository = Arc::new(FakeOrderRepository {
            inserted: Mutex::new(Vec::new()),
            fail_insert: true,
        });
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = PlaceOrderFromCart::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:00 UTC),
            }),
            Arc::new(FixedIdGenerator),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(PlaceOrderFromCartInput {
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                items: vec![PlaceOrderItemInput {
                    catalog_item_id: "item-1".to_string(),
                    name: "Fried Rice".to_string(),
                    unit_price_amount: 3200,
                    quantity: 1,
                }],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
        assert!(event_recorder.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn place_order_rolls_back_when_record_fails() {
        let repository = Arc::new(FakeOrderRepository::default());
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder {
            events: Mutex::new(Vec::new()),
            fail_record: true,
        });
        let use_case = PlaceOrderFromCart::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:00 UTC),
            }),
            Arc::new(FixedIdGenerator),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(PlaceOrderFromCartInput {
                customer_id: "customer-1".to_string(),
                store_id: "store-1".to_string(),
                items: vec![PlaceOrderItemInput {
                    catalog_item_id: "item-1".to_string(),
                    name: "Fried Rice".to_string(),
                    unit_price_amount: 3200,
                    quantity: 1,
                }],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        assert_eq!(repository.inserted.lock().unwrap().len(), 1);
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
        assert!(event_recorder.events.lock().unwrap().is_empty());
    }
}
