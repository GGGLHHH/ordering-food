use crate::{ApplicationError, Clock, IdGenerator, OrganizationUnitOfWorkFactory};
use ordering_food_organization_domain::{BrandId, OrganizationStatus, Store};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStoreInput {
    pub brand_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
}

pub struct CreateStore {
    unit_of_work_factory: Arc<dyn OrganizationUnitOfWorkFactory>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateStore {
    pub fn new(
        unit_of_work_factory: Arc<dyn OrganizationUnitOfWorkFactory>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            unit_of_work_factory,
            clock,
            id_generator,
        }
    }

    pub async fn execute(&self, input: CreateStoreInput) -> Result<Store, ApplicationError> {
        let now = self.clock.now();
        let brand_id = BrandId::new(input.brand_id);
        let status = OrganizationStatus::parse(input.status)?;
        let mut unit_of_work = self.unit_of_work_factory.begin().await?;

        let brand_exists = match unit_of_work.find_brand_by_id(&brand_id).await {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(error) => {
                unit_of_work.rollback().await?;
                return Err(error);
            }
        };
        if !brand_exists {
            unit_of_work.rollback().await?;
            return Err(ApplicationError::not_found("brand was not found"));
        }

        let store = match Store::create(
            self.id_generator.next_store_id(),
            brand_id,
            input.slug,
            input.name,
            input.currency_code,
            input.timezone,
            status,
            now,
        ) {
            Ok(store) => store,
            Err(error) => {
                unit_of_work.rollback().await?;
                return Err(error.into());
            }
        };

        if let Err(error) = unit_of_work.insert_store(&store).await {
            unit_of_work.rollback().await?;
            return Err(error);
        }

        unit_of_work.commit().await?;
        Ok(store)
    }
}
