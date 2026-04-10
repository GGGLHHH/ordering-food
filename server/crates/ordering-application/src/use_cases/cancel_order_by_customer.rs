use crate::{
    ApplicationError, Clock, LocalCommercialOrderCancelledByCustomer,
    LocalCommercialOrderStatusChanged, OrderRepository, OrderingEvent, OrderingEventRecorder,
    TransactionManager,
};
use ordering_food_ordering_domain::OrderId;
use ordering_food_shared_kernel::Identifier;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelOrderByCustomerInput {
    pub order_id: String,
    pub customer_id: String,
}

pub struct CancelOrderByCustomer {
    order_repository: Arc<dyn OrderRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    event_recorder: Arc<dyn OrderingEventRecorder>,
}

impl CancelOrderByCustomer {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        event_recorder: Arc<dyn OrderingEventRecorder>,
    ) -> Self {
        Self {
            order_repository,
            transaction_manager,
            clock,
            event_recorder,
        }
    }

    pub async fn execute(&self, input: CancelOrderByCustomerInput) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let order_id = OrderId::new(input.order_id);
        let order = match self.order_repository.find_by_id(tx.as_mut(), &order_id).await {
            Ok(order) => order,
            Err(error) => {
                self.transaction_manager.rollback(tx).await?;
                return Err(error);
            }
        };
        let mut order = match order {
            Some(order) => order,
            None => {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::not_found("order was not found"));
            }
        };

        if order.customer_id().as_str() != input.customer_id {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("order was not found"));
        }

        let previous_status = order.status().as_str().to_string();
        if let Err(error) = order.cancel_by_customer(self.clock.now()) {
            self.transaction_manager.rollback(tx).await?;
            return Err(error.into());
        }

        if let Err(error) = self.order_repository.update(tx.as_mut(), &order).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let state_changed = OrderingEvent::CommercialOrderStatusChanged(
            LocalCommercialOrderStatusChanged {
                order_id: order_id.as_str().to_string(),
                customer_id: order.customer_id().as_str().to_string(),
                store_id: order.store_id().as_str().to_string(),
                previous_status,
                current_status: order.status().as_str().to_string(),
                occurred_at: order.updated_at(),
            },
        );
        if let Err(error) = self.event_recorder.record(tx.as_mut(), &state_changed).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let cancelled = OrderingEvent::CommercialOrderCancelledByCustomer(
            LocalCommercialOrderCancelledByCustomer {
                order_id: order_id.as_str().to_string(),
                customer_id: order.customer_id().as_str().to_string(),
                store_id: order.store_id().as_str().to_string(),
                occurred_at: order.updated_at(),
            },
        );
        if let Err(error) = self.event_recorder.record(tx.as_mut(), &cancelled).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OrderingEvent, OrderingEventRecorder, TransactionContext};
    use async_trait::async_trait;
    use ordering_food_ordering_domain::{
        CatalogItemId, CustomerId, Order, OrderStatus, PlaceOrderItemInput, StoreId,
    };
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

    #[derive(Default)]
    struct RecordingEventRecorder {
        events: Mutex<Vec<OrderingEvent>>,
    }

    #[async_trait]
    impl OrderingEventRecorder for RecordingEventRecorder {
        async fn record(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &OrderingEvent,
        ) -> Result<(), ApplicationError> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct FailingRecordingEventRecorder {
        events: Mutex<Vec<OrderingEvent>>,
        call_count: Mutex<usize>,
        fail_on_call: Option<usize>,
    }

    impl FailingRecordingEventRecorder {
        fn new(fail_on_call: Option<usize>) -> Self {
            Self {
                events: Mutex::new(Vec::new()),
                call_count: Mutex::new(0),
                fail_on_call,
            }
        }

        fn failing_on(call_index: usize) -> Self {
            Self::new(Some(call_index))
        }

        fn recorded_events(&self) -> Vec<OrderingEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl OrderingEventRecorder for FailingRecordingEventRecorder {
        async fn record(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &OrderingEvent,
        ) -> Result<(), ApplicationError> {
            let call_index = {
                let mut guard = self.call_count.lock().unwrap();
                *guard += 1;
                *guard
            };

            if self.fail_on_call == Some(call_index) {
                return Err(ApplicationError::unexpected(format!(
                    "record failure triggered for call {}",
                    call_index
                )));
            }

            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    struct FakeOrderRepository {
        order: Mutex<Option<Order>>,
        fail_find: bool,
        fail_update: bool,
    }

    impl FakeOrderRepository {
        fn with_order(order: Order) -> Self {
            Self {
                order: Mutex::new(Some(order)),
                fail_find: false,
                fail_update: false,
            }
        }

        fn without_order() -> Self {
            Self {
                order: Mutex::new(None),
                fail_find: false,
                fail_update: false,
            }
        }
    }

    #[async_trait]
    impl crate::OrderRepository for FakeOrderRepository {
        async fn find_by_id(
            &self,
            _tx: &mut dyn TransactionContext,
            _order_id: &OrderId,
        ) -> Result<Option<Order>, ApplicationError> {
            if self.fail_find {
                return Err(ApplicationError::unexpected("find failed"));
            }
            Ok(self.order.lock().unwrap().clone())
        }

        async fn insert(
            &self,
            _tx: &mut dyn TransactionContext,
            _order: &Order,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn update(
            &self,
            _tx: &mut dyn TransactionContext,
            order: &Order,
        ) -> Result<(), ApplicationError> {
            if self.fail_update {
                return Err(ApplicationError::unexpected("update failed"));
            }
            *self.order.lock().unwrap() = Some(order.clone());
            Ok(())
        }
    }

    fn make_order(status: OrderStatus) -> Order {
        let mut order = Order::place(
            OrderId::new("order-1"),
            CustomerId::new("customer-1"),
            StoreId::new("store-1"),
            vec![PlaceOrderItemInput {
                catalog_item_id: CatalogItemId::new("item-1"),
                name: "Noodles".to_string(),
                unit_price_amount: 1800,
                quantity: 1,
            }],
            datetime!(2026-03-15 10:00 UTC),
        )
        .unwrap();

        if status == OrderStatus::CancelledByCustomer {
            order
                .cancel_by_customer(datetime!(2026-03-15 10:01 UTC))
                .unwrap();
        }

        order
    }

    #[tokio::test]
    async fn cancel_order_returns_not_found_for_non_owner() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::Placed,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "other-user".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::NotFound { .. }));
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
        assert!(event_recorder.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn cancel_order_updates_status_for_owner() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::Placed,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = CancelOrderByCustomer::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            repository.order.lock().unwrap().as_ref().unwrap().status(),
            OrderStatus::CancelledByCustomer
        );
        assert_eq!(*transactions.committed.lock().unwrap(), 1);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 0);
        let events = event_recorder.events.lock().unwrap();
        assert_eq!(events.len(), 2);

        let status_changed = match &events[0] {
            OrderingEvent::CommercialOrderStatusChanged(event) => event,
            _ => panic!("expected CommercialOrderStatusChanged as first event"),
        };
        assert_eq!(status_changed.order_id, "order-1");
        assert_eq!(status_changed.customer_id, "customer-1");
        assert_eq!(status_changed.store_id, "store-1");
        assert_eq!(status_changed.previous_status, "placed");
        assert_eq!(status_changed.current_status, "cancelled_by_customer");
        assert_eq!(status_changed.occurred_at, datetime!(2026-03-15 10:05 UTC));

        let cancelled = match &events[1] {
            OrderingEvent::CommercialOrderCancelledByCustomer(event) => event,
            _ => panic!("expected CommercialOrderCancelledByCustomer as second event"),
        };
        assert_eq!(cancelled.order_id, "order-1");
        assert_eq!(cancelled.customer_id, "customer-1");
        assert_eq!(cancelled.store_id, "store-1");
        assert_eq!(cancelled.occurred_at, datetime!(2026-03-15 10:05 UTC));
    }

    #[tokio::test]
    async fn cancel_order_returns_conflict_for_cancelled_order() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::CancelledByCustomer,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Conflict { .. }));
        assert!(event_recorder.events.lock().unwrap().is_empty());
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn cancel_order_rolls_back_when_order_not_found() {
        let repository = Arc::new(FakeOrderRepository::without_order());
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "missing-order".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::NotFound { .. }));
        assert!(event_recorder.events.lock().unwrap().is_empty());
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn cancel_order_rolls_back_when_lookup_fails() {
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(None),
            fail_find: true,
            fail_update: false,
        });
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        assert!(event_recorder.events.lock().unwrap().is_empty());
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn cancel_order_rolls_back_when_update_fails() {
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(make_order(OrderStatus::Placed))),
            fail_find: false,
            fail_update: true,
        });
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(RecordingEventRecorder::default());
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        assert!(event_recorder.events.lock().unwrap().is_empty());
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn cancel_order_rolls_back_when_first_event_record_fails() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::Placed,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(FailingRecordingEventRecorder::failing_on(1));
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        assert_eq!(event_recorder.recorded_events().len(), 0);
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn cancel_order_rolls_back_when_second_event_record_fails() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::Placed,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let event_recorder = Arc::new(FailingRecordingEventRecorder::failing_on(2));
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
            event_recorder.clone(),
        );

        let error = use_case
            .execute(CancelOrderByCustomerInput {
                order_id: "order-1".to_string(),
                customer_id: "customer-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(error, ApplicationError::Unexpected { .. }));
        let events = event_recorder.recorded_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            OrderingEvent::CommercialOrderStatusChanged(event) => {
                assert_eq!(event.order_id, "order-1");
                assert_eq!(event.customer_id, "customer-1");
            }
            _ => panic!("expected CommercialOrderStatusChanged as first event"),
        }
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 1);
    }
}
