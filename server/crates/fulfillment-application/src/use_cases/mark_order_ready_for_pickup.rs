use crate::{
    ApplicationError, Clock, CommercialOrderProjectionQueryService, TransactionManager,
    WorkflowAction, WorkflowActionAuthorizer, WorkflowOrderRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkOrderReadyForPickupInput {
    pub order_id: String,
    pub actor_user_id: String,
}

pub struct MarkOrderReadyForPickup {
    workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
    workflow_action_authorizer: Arc<dyn WorkflowActionAuthorizer>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    commercial_order_projection_queries: Arc<CommercialOrderProjectionQueryService>,
}

impl MarkOrderReadyForPickup {
    pub fn new(
        workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
        workflow_action_authorizer: Arc<dyn WorkflowActionAuthorizer>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        commercial_order_projection_queries: Arc<CommercialOrderProjectionQueryService>,
    ) -> Self {
        Self {
            workflow_order_repository,
            workflow_action_authorizer,
            transaction_manager,
            clock,
            commercial_order_projection_queries,
        }
    }

    pub async fn execute(
        &self,
        input: MarkOrderReadyForPickupInput,
    ) -> Result<(), ApplicationError> {
        self.commercial_order_projection_queries
            .ensure_workflow_transition_allowed(&input.order_id)
            .await?;

        let mut tx = self.transaction_manager.begin().await?;
        let mut workflow_order = self
            .workflow_order_repository
            .find_by_ordering_order_id(tx.as_mut(), &input.order_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("workflow order was not found"))?;

        if let Err(error) = self
            .workflow_action_authorizer
            .ensure_actor_can_manage_order(
                &input.actor_user_id,
                workflow_order.store_id(),
                WorkflowAction::MarkReady,
            )
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        workflow_order.mark_ready(self.clock.now())?;

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
