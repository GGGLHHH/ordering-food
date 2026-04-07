use crate::{
    SqlxBrandCatalogReadRepository, SqlxBrandCatalogRepository, SqlxCategoryReadRepository,
    SqlxCategoryRepository, SqlxItemReadRepository, SqlxItemRepository,
    SqlxStoreCatalogReadRepository, SqlxStoreCatalogRepository, SqlxStoreItemListingRepository,
    SqlxTransactionManager,
};
use ordering_food_catalog_application::{
    CatalogModule, Clock, IdGenerator, OrganizationScopeReader,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct CatalogSqlxModule {
    application: Arc<CatalogModule>,
    brand_catalog_reads: Arc<SqlxBrandCatalogReadRepository>,
    store_catalog_reads: Arc<SqlxStoreCatalogReadRepository>,
    category_reads: Arc<SqlxCategoryReadRepository>,
    item_reads: Arc<SqlxItemReadRepository>,
}

impl CatalogSqlxModule {
    pub fn application(&self) -> Arc<CatalogModule> {
        self.application.clone()
    }

    pub fn brand_catalog_reads(&self) -> Arc<SqlxBrandCatalogReadRepository> {
        self.brand_catalog_reads.clone()
    }

    pub fn store_catalog_reads(&self) -> Arc<SqlxStoreCatalogReadRepository> {
        self.store_catalog_reads.clone()
    }

    pub fn category_reads(&self) -> Arc<SqlxCategoryReadRepository> {
        self.category_reads.clone()
    }

    pub fn item_reads(&self) -> Arc<SqlxItemReadRepository> {
        self.item_reads.clone()
    }
}

pub fn build_catalog_sqlx_module(
    pool: PgPool,
    organization_scope_reader: Arc<dyn OrganizationScopeReader>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
) -> CatalogSqlxModule {
    let brand_catalog_reads = Arc::new(SqlxBrandCatalogReadRepository::new(pool.clone()));
    let store_catalog_reads = Arc::new(SqlxStoreCatalogReadRepository::new(pool.clone()));
    let category_reads = Arc::new(SqlxCategoryReadRepository::new(pool.clone()));
    let item_reads = Arc::new(SqlxItemReadRepository::new(pool.clone()));

    let application = Arc::new(CatalogModule::new(
        organization_scope_reader,
        Arc::new(SqlxBrandCatalogRepository),
        Arc::new(SqlxStoreCatalogRepository),
        Arc::new(SqlxCategoryRepository),
        Arc::new(SqlxItemRepository),
        Arc::new(SqlxStoreItemListingRepository),
        brand_catalog_reads.clone(),
        store_catalog_reads.clone(),
        category_reads.clone(),
        item_reads.clone(),
        item_reads.clone(),
        Arc::new(SqlxTransactionManager::new(pool)),
        clock,
        id_generator,
    ));

    CatalogSqlxModule {
        application,
        brand_catalog_reads,
        store_catalog_reads,
        category_reads,
        item_reads,
    }
}
