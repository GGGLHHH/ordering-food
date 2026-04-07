use crate::{
    ApplicationError, Clock, OrderCancelledByCustomer, OrderCommercialStateChanged,
    OrderRepository, OrderingPublishedEventRecorder, TransactionManager,
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
    event_recorder: Arc<dyn OrderingPublishedEventRecorder>,
}

impl CancelOrderByCustomer {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        event_recorder: Arc<dyn OrderingPublishedEventRecorder>,
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
        let mut order = self
            .order_repository
            .find_by_id(tx.as_mut(), &order_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("order was not found"))?;

        if order.customer_id().as_str() != input.customer_id {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::not_found("order was not found"));
        }

        let previous_status = order.status().as_str().to_string();
        order.cancel_by_customer(self.clock.now())?;

        if let Err(error) = self.order_repository.update(tx.as_mut(), &order).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let state_changed = OrderCommercialStateChanged {
            order_id: order_id.as_str().to_string(),
            customer_id: order.customer_id().as_str().to_string(),
            store_id: order.store_id().as_str().to_string(),
            previous_status,
            current_status: order.status().as_str().to_string(),
            occurred_at: order.updated_at(),
        };
        if let Err(error) = self
            .event_recorder
            .record_order_commercial_state_changed(tx.as_mut(), &state_changed)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let cancelled = OrderCancelledByCustomer {
            order_id: order_id.as_str().to_string(),
            customer_id: order.customer_id().as_str().to_string(),
            store_id: order.store_id().as_str().to_string(),
            occurred_at: order.updated_at(),
        };
        if let Err(error) = self
            .event_recorder
            .record_order_cancelled_by_customer(tx.as_mut(), &cancelled)
            .await
        {
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
    use crate::{OrderingPublishedEventRecorder, TransactionContext};
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
        commercial_state_changes: Mutex<Vec<crate::OrderCommercialStateChanged>>,
        cancelled: Mutex<Vec<crate::OrderCancelledByCustomer>>,
    }

    #[async_trait]
    impl OrderingPublishedEventRecorder for RecordingEventRecorder {
        async fn record_order_placed(
            &self,
            _tx: &mut dyn TransactionContext,
            _event: &crate::OrderPlaced,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn record_order_commercial_state_changed(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &crate::OrderCommercialStateChanged,
        ) -> Result<(), ApplicationError> {
            self.commercial_state_changes
                .lock()
                .unwrap()
                .push(event.clone());
            Ok(())
        }

        async fn record_order_cancelled_by_customer(
            &self,
            _tx: &mut dyn TransactionContext,
            event: &crate::OrderCancelledByCustomer,
        ) -> Result<(), ApplicationError> {
            self.cancelled.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    struct FakeOrderRepository {
        order: Mutex<Option<Order>>,
    }

    impl FakeOrderRepository {
        fn with_order(order: Order) -> Self {
            Self {
                order: Mutex::new(Some(order)),
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
        assert!(event_recorder.cancelled.lock().unwrap().is_empty());
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
        assert_eq!(event_recorder.cancelled.lock().unwrap().len(), 1);
        assert_eq!(
            event_recorder
                .commercial_state_changes
                .lock()
                .unwrap()
                .len(),
            1
        );
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
        assert!(event_recorder.cancelled.lock().unwrap().is_empty());
        assert_eq!(*transactions.committed.lock().unwrap(), 0);
        assert_eq!(*transactions.rolled_back.lock().unwrap(), 0);
    }
}
