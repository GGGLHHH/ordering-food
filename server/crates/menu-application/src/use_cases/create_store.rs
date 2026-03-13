use crate::{ApplicationError, Clock, IdGenerator, StoreRepository, TransactionManager};
use ordering_food_menu_domain::{MenuStatus, Store};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStoreInput {
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
}

pub struct CreateStore {
    store_repository: Arc<dyn StoreRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateStore {
    pub fn new(
        store_repository: Arc<dyn StoreRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            store_repository,
            transaction_manager,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateStoreInput) -> Result<Store, ApplicationError> {
        let now = self.clock.now();
        let store = Store::create(
            self.id_generator.next_store_id(),
            input.slug,
            input.name,
            input.currency_code,
            input.timezone,
            MenuStatus::parse(input.status)?,
            now,
        )?;
        let mut tx = self.transaction_manager.begin().await?;

        if let Err(error) = self.store_repository.insert(tx.as_mut(), &store).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await?;
        Ok(store)
    }
}
