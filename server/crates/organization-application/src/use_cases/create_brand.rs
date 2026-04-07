use crate::{ApplicationError, Clock, IdGenerator, OrganizationUnitOfWorkFactory};
use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateBrandInput {
    pub brand_id: Option<String>,
    pub slug: String,
    pub name: String,
    pub status: String,
}

pub struct CreateBrand {
    unit_of_work_factory: Arc<dyn OrganizationUnitOfWorkFactory>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateBrand {
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

    pub async fn execute(&self, input: CreateBrandInput) -> Result<String, ApplicationError> {
        let now = self.clock.now();
        let brand_id = input
            .brand_id
            .map(BrandId::new)
            .unwrap_or_else(|| self.id_generator.next_brand_id());
        let brand = Brand::create(
            brand_id,
            input.slug,
            input.name,
            OrganizationStatus::parse(input.status)?,
            now,
        )?;
        let mut unit_of_work = self.unit_of_work_factory.begin().await?;

        if let Err(error) = unit_of_work.insert_brand(&brand).await {
            unit_of_work.rollback().await?;
            return Err(error);
        }

        unit_of_work.commit().await?;
        Ok(brand.id().as_str().to_string())
    }
}
