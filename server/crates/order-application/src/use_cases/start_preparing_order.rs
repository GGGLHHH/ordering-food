use crate::{ApplicationError, Clock, OrderRepository, TransactionManager};
use ordering_food_order_domain::OrderId;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartPreparingOrderInput {
    pub order_id: String,
    pub actor_user_id: String,
}

pub struct StartPreparingOrder {
    order_repository: Arc<dyn OrderRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
}

impl StartPreparingOrder {
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

    pub async fn execute(&self, input: StartPreparingOrderInput) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let order_id = OrderId::new(input.order_id);
        let mut order = self
            .order_repository
            .find_by_id(tx.as_mut(), &order_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("order was not found"))?;

        order.start_preparing(self.clock.now())?;

        if let Err(error) = self.order_repository.update(tx.as_mut(), &order).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await
    }
}
