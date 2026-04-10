use crate::{
    ApplicationError, CommercialOrderCancelledByCustomer, CommercialOrderPlaced,
    CommercialOrderProjectionItemReadModel, CommercialOrderProjectionReadModel,
    CommercialOrderProjectionStore, CommercialOrderStateChanged, IdGenerator, TransactionManager,
    WorkflowOrderRepository,
};
use ordering_food_fulfillment_domain::{FulfillmentOrder, WorkflowStatus};
use std::sync::Arc;

pub struct OrderingCommercialEventHandler {
    workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
    commercial_order_projection_store: Arc<dyn CommercialOrderProjectionStore>,
    transaction_manager: Arc<dyn TransactionManager>,
    id_generator: Arc<dyn IdGenerator>,
}

impl OrderingCommercialEventHandler {
    pub fn new(
        workflow_order_repository: Arc<dyn WorkflowOrderRepository>,
        commercial_order_projection_store: Arc<dyn CommercialOrderProjectionStore>,
        transaction_manager: Arc<dyn TransactionManager>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            workflow_order_repository,
            commercial_order_projection_store,
            transaction_manager,
            id_generator,
        }
    }

    pub async fn handle_order_placed(
        &self,
        event: &CommercialOrderPlaced,
    ) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let projection = map_order_placed_projection(event);

        if let Err(error) = self
            .commercial_order_projection_store
            .upsert(tx.as_mut(), &projection)
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let existing = match self
            .workflow_order_repository
            .find_by_ordering_order_id(tx.as_mut(), &event.order_id)
            .await
        {
            Ok(existing) => existing,
            Err(error) => {
                self.transaction_manager.rollback(tx).await?;
                return Err(error);
            }
        };

        if existing.is_none() {
            let workflow_order = FulfillmentOrder::bootstrap(
                self.id_generator.next_fulfillment_order_id().as_str(),
                event.order_id.clone(),
                event.store_id.clone(),
                event.created_at,
            );

            if let Err(error) = self
                .workflow_order_repository
                .insert(tx.as_mut(), &workflow_order)
                .await
            {
                self.transaction_manager.rollback(tx).await?;
                return Err(error);
            }
        }

        self.transaction_manager.commit(tx).await
    }

    pub async fn handle_order_commercial_state_changed(
        &self,
        event: &CommercialOrderStateChanged,
    ) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;

        if let Err(error) = self
            .commercial_order_projection_store
            .update_status(
                tx.as_mut(),
                &event.order_id,
                &event.current_status,
                event.occurred_at,
            )
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await
    }

    pub async fn handle_order_cancelled_by_customer(
        &self,
        event: &CommercialOrderCancelledByCustomer,
    ) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;

        if let Err(error) = self
            .commercial_order_projection_store
            .update_status(
                tx.as_mut(),
                &event.order_id,
                "cancelled_by_customer",
                event.occurred_at,
            )
            .await
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        let Some(mut workflow_order) = self
            .workflow_order_repository
            .find_by_ordering_order_id(tx.as_mut(), &event.order_id)
            .await?
        else {
            return self.transaction_manager.commit(tx).await;
        };

        if workflow_order.status() != WorkflowStatus::CancelledByCustomer {
            if let Err(error) = workflow_order.cancel_by_customer(event.occurred_at) {
                self.transaction_manager.rollback(tx).await?;
                return Err(error.into());
            }

            if let Err(error) = self
                .workflow_order_repository
                .update(tx.as_mut(), &workflow_order)
                .await
            {
                self.transaction_manager.rollback(tx).await?;
                return Err(error);
            }
        }

        self.transaction_manager.commit(tx).await
    }
}

fn map_order_placed_projection(
    event: &CommercialOrderPlaced,
) -> CommercialOrderProjectionReadModel {
    CommercialOrderProjectionReadModel {
        order_id: event.order_id.clone(),
        customer_id: event.customer_id.clone(),
        store_id: event.store_id.clone(),
        status: event.status.clone(),
        subtotal_amount: event.subtotal_amount,
        total_amount: event.total_amount,
        created_at: event.created_at,
        updated_at: event.updated_at,
        items: event
            .items
            .iter()
            .map(|item| CommercialOrderProjectionItemReadModel {
                line_number: item.line_number,
                catalog_item_id: item.catalog_item_id.clone(),
                name: item.name.clone(),
                unit_price_amount: item.unit_price_amount,
                quantity: item.quantity,
                line_total_amount: item.line_total_amount,
            })
            .collect(),
    }
}
