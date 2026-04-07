use crate::{SqlxBrandReadRepository, SqlxOrganizationUnitOfWorkFactory, SqlxStoreReadRepository};
use ordering_food_organization_application::{Clock, IdGenerator, OrganizationModule};
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_organization_module(
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
) -> Arc<OrganizationModule> {
    let unit_of_work_factory = Arc::new(SqlxOrganizationUnitOfWorkFactory::new(pool.clone()));
    let brand_read_repository = Arc::new(SqlxBrandReadRepository::new(pool.clone()));
    let store_read_repository = Arc::new(SqlxStoreReadRepository::new(pool));

    Arc::new(OrganizationModule::new(
        unit_of_work_factory,
        brand_read_repository,
        store_read_repository,
        clock,
        id_generator,
    ))
}
