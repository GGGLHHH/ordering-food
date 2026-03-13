use crate::{ApplicationError, CategoryReadModel, ItemListFilter, ItemReadModel, StoreReadModel};
use async_trait::async_trait;
use ordering_food_menu_domain::{Category, CategoryId, Item, ItemId, Store, StoreId};
use ordering_food_shared_kernel::Timestamp;
use std::{any::Any, sync::Arc};

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

pub trait IdGenerator: Send + Sync {
    fn next_store_id(&self) -> StoreId;
    fn next_category_id(&self) -> CategoryId;
    fn next_item_id(&self) -> ItemId;
}

pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError>;
    async fn commit(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
    async fn rollback(&self, tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait StoreRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_id: &StoreId,
    ) -> Result<Option<Store>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        store: &Store,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        category_id: &CategoryId,
    ) -> Result<Option<Category>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        category: &Category,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        item: &Item,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait StoreReadRepository: Send + Sync {
    async fn get_active(&self) -> Result<Option<StoreReadModel>, ApplicationError>;
}

#[async_trait]
pub trait CategoryReadRepository: Send + Sync {
    async fn list_active_by_store(
        &self,
        store_id: &StoreId,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError>;

    async fn get_active_by_slug(
        &self,
        store_id: &StoreId,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError>;
}

#[async_trait]
pub trait ItemReadRepository: Send + Sync {
    async fn list_active_by_store(
        &self,
        store_id: &StoreId,
        filter: ItemListFilter,
    ) -> Result<Vec<ItemReadModel>, ApplicationError>;

    async fn get_active_by_id(
        &self,
        item_id: &ItemId,
    ) -> Result<Option<ItemReadModel>, ApplicationError>;
}

#[derive(Clone)]
pub struct StoreQueryService {
    repository: Arc<dyn StoreReadRepository>,
}

impl StoreQueryService {
    pub fn new(repository: Arc<dyn StoreReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_active(&self) -> Result<Option<StoreReadModel>, ApplicationError> {
        self.repository.get_active().await
    }
}

#[derive(Clone)]
pub struct CategoryQueryService {
    repository: Arc<dyn CategoryReadRepository>,
}

impl CategoryQueryService {
    pub fn new(repository: Arc<dyn CategoryReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_active_by_store(
        &self,
        store_id: &StoreId,
    ) -> Result<Vec<CategoryReadModel>, ApplicationError> {
        self.repository.list_active_by_store(store_id).await
    }

    pub async fn get_active_by_slug(
        &self,
        store_id: &StoreId,
        slug: &str,
    ) -> Result<Option<CategoryReadModel>, ApplicationError> {
        self.repository.get_active_by_slug(store_id, slug).await
    }
}

#[derive(Clone)]
pub struct ItemQueryService {
    repository: Arc<dyn ItemReadRepository>,
}

impl ItemQueryService {
    pub fn new(repository: Arc<dyn ItemReadRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_active_by_store(
        &self,
        store_id: &StoreId,
        filter: ItemListFilter,
    ) -> Result<Vec<ItemReadModel>, ApplicationError> {
        self.repository.list_active_by_store(store_id, filter).await
    }

    pub async fn get_active_by_id(
        &self,
        item_id: &ItemId,
    ) -> Result<Option<ItemReadModel>, ApplicationError> {
        self.repository.get_active_by_id(item_id).await
    }
}
