use crate::{
    CategoryQueryService, CategoryReadRepository, CategoryRepository, Clock, CreateCategory,
    CreateItem, CreateStore, IdGenerator, ItemQueryService, ItemReadRepository, ItemRepository,
    StoreQueryService, StoreReadRepository, StoreRepository, TransactionManager,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MenuModule {
    pub create_store: Arc<CreateStore>,
    pub create_category: Arc<CreateCategory>,
    pub create_item: Arc<CreateItem>,
    pub store_queries: Arc<StoreQueryService>,
    pub category_queries: Arc<CategoryQueryService>,
    pub item_queries: Arc<ItemQueryService>,
}

impl MenuModule {
    pub fn new(
        store_repository: Arc<dyn StoreRepository>,
        category_repository: Arc<dyn CategoryRepository>,
        item_repository: Arc<dyn ItemRepository>,
        store_read_repository: Arc<dyn StoreReadRepository>,
        category_read_repository: Arc<dyn CategoryReadRepository>,
        item_read_repository: Arc<dyn ItemReadRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            create_store: Arc::new(CreateStore::new(
                store_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
                id_generator.clone(),
            )),
            create_category: Arc::new(CreateCategory::new(
                store_repository.clone(),
                category_repository.clone(),
                transaction_manager.clone(),
                clock.clone(),
                id_generator.clone(),
            )),
            create_item: Arc::new(CreateItem::new(
                store_repository,
                category_repository,
                item_repository,
                transaction_manager,
                clock,
                id_generator,
            )),
            store_queries: Arc::new(StoreQueryService::new(store_read_repository)),
            category_queries: Arc::new(CategoryQueryService::new(category_read_repository)),
            item_queries: Arc::new(ItemQueryService::new(item_read_repository)),
        }
    }
}
