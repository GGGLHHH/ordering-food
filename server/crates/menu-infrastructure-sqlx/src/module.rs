use crate::{
    SqlxCategoryReadRepository, SqlxCategoryRepository, SqlxItemReadRepository, SqlxItemRepository,
    SqlxStoreReadRepository, SqlxStoreRepository, SqlxTransactionManager,
};
use ordering_food_menu_application::{Clock, IdGenerator, MenuModule};
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_menu_module(
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
) -> Arc<MenuModule> {
    let store_repository = Arc::new(SqlxStoreRepository);
    let category_repository = Arc::new(SqlxCategoryRepository);
    let item_repository = Arc::new(SqlxItemRepository);
    let store_read_repository = Arc::new(SqlxStoreReadRepository::new(pool.clone()));
    let category_read_repository = Arc::new(SqlxCategoryReadRepository::new(pool.clone()));
    let item_read_repository = Arc::new(SqlxItemReadRepository::new(pool.clone()));
    let transaction_manager = Arc::new(SqlxTransactionManager::new(pool));

    Arc::new(MenuModule::new(
        store_repository,
        category_repository,
        item_repository,
        store_read_repository,
        category_read_repository,
        item_read_repository,
        transaction_manager,
        clock,
        id_generator,
    ))
}
