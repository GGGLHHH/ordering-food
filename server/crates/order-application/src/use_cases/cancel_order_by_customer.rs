use crate::{ApplicationError, Clock, OrderRepository, TransactionManager};
use ordering_food_order_domain::OrderId;
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
}

impl CancelOrderByCustomer {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            order_repository,
            transaction_manager,
            clock,
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

        order.cancel_by_customer(self.clock.now())?;

        if let Err(error) = self.order_repository.update(tx.as_mut(), &order).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionContext;
    use async_trait::async_trait;
    use ordering_food_order_domain::{
        CustomerId, MenuItemId, Order, OrderStatus, PlaceOrderItemInput, StoreId,
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
                menu_item_id: MenuItemId::new("item-1"),
                name: "Noodles".to_string(),
                unit_price_amount: 1800,
                quantity: 1,
            }],
            datetime!(2026-03-15 10:00 UTC),
        )
        .unwrap();

        if status == OrderStatus::Accepted {
            order.accept(datetime!(2026-03-15 10:01 UTC)).unwrap();
        }

        order
    }

    #[tokio::test]
    async fn cancel_order_returns_not_found_for_non_owner() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::PendingAcceptance,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let use_case = CancelOrderByCustomer::new(
            repository,
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
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
    }

    #[tokio::test]
    async fn cancel_order_updates_status_for_owner() {
        let repository = Arc::new(FakeOrderRepository::with_order(make_order(
            OrderStatus::Accepted,
        )));
        let transactions = Arc::new(RecordingTransactionManager::default());
        let use_case = CancelOrderByCustomer::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FixedClock {
                now: datetime!(2026-03-15 10:05 UTC),
            }),
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
    }
}
