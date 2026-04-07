use crate::{
    ApplicationError, Clock, IdGenerator, OrderPlaced, OrderRepository,
    OrderingPublishedEventRecorder, TransactionManager,
};
use ordering_food_ordering_domain::{
    CatalogItemId, CustomerId, Order, PlaceOrderItemInput as DomainPlaceOrderItemInput, StoreId,
};
use ordering_food_ordering_published::OrderPlacedItem;
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
    event_recorder: Arc<dyn OrderingPublishedEventRecorder>,
}

impl PlaceOrderFromCart {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        event_recorder: Arc<dyn OrderingPublishedEventRecorder>,
    ) -> Self {
        Self {
            order_repository,
            transaction_manager,
            clock,
            id_generator,
            event_recorder,
        }
    }

    pub async fn execute(&self, input: PlaceOrderFromCartInput) -> Result<String, ApplicationError> {
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

        let order_placed = OrderPlaced {
            order_id: order.id().as_str().to_string(),
            customer_id: order.customer_id().as_str().to_string(),
            store_id: order.store_id().as_str().to_string(),
            status: order.status().as_str().to_string(),
            subtotal_amount: order.subtotal_amount(),
            total_amount: order.total_amount(),
            created_at: order.created_at(),
            updated_at: order.updated_at(),
            items: order
                .items()
                .iter()
                .map(|item| OrderPlacedItem {
                    line_number: item.line_number(),
                    catalog_item_id: item.catalog_item_id().as_str().to_string(),
                    name: item.name().to_string(),
                    unit_price_amount: item.unit_price_amount(),
                    quantity: item.quantity(),
                    line_total_amount: item.line_total_amount(),
                })
                .collect(),
        };

        if let Err(error) = self
            .event_recorder
            .record_order_placed(tx.as_mut(), &order_placed)
            .await
        {
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
        ApplicationError, OrderPlaced, OrderRepository, OrderingPublishedEventRecorder,
        TransactionContext,
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
        placed: Mutex<Vec<OrderPlaced>>,
    }

    #[async_trait]
    impl OrderingPublishedEventRecorder for RecordingEventRecorder {
        async fn record_order_placed(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &OrderPlaced,
        ) -> Result<(), ApplicationError> {
            self.placed.lock().unwrap().push(event.clone());
            Ok(())
        }

        async fn record_order_commercial_state_changed(
            &self,
            _tx: &mut dyn TransactionContext,
            _event: &crate::OrderCommercialStateChanged,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn record_order_cancelled_by_customer(
            &self,
            _tx: &mut dyn TransactionContext,
            _event: &crate::OrderCancelledByCustomer,
        ) -> Result<(), ApplicationError> {
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
        assert_eq!(event_recorder.placed.lock().unwrap().len(), 1);
        assert_eq!(
            event_recorder.placed.lock().unwrap()[0].store_id,
            "store-1".to_string()
        );
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
        assert!(event_recorder.placed.lock().unwrap().is_empty());
    }
}
