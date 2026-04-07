use crate::{
    ApplicationError, Clock, CommercialOrderProjectionQueryService, TransactionManager,
    WorkflowOrderRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectOrderByStoreInput {
    pub order_id: String,
    pub actor_user_id: String,
}

pub struct RejectOrderByStore {
    workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    commercial_order_projection_queries: Arc<CommercialOrderProjectionQueryService>,
}

impl RejectOrderByStore {
    pub fn new(
        workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        commercial_order_projection_queries: Arc<CommercialOrderProjectionQueryService>,
    ) -> Self {
        Self {
            workflow_order_repository,
            transaction_manager,
            clock,
            commercial_order_projection_queries,
        }
    }

    pub async fn execute(&self, input: RejectOrderByStoreInput) -> Result<(), ApplicationError> {
        self.commercial_order_projection_queries
            .ensure_workflow_transition_allowed(&input.order_id)
            .await?;

        let mut tx = self.transaction_manager.begin().await?;
        let mut workflow_order = self
            .workflow_order_repository
            .find_by_ordering_order_id(tx.as_mut(), &input.order_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("workflow order was not found"))?;

        workflow_order.reject_by_store(self.clock.now())?;

        if let Err(error) = self
            .workflow_order_repository
            .update(tx.as_mut(), &workflow_order)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await
    }
}
