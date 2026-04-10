mod dto;
mod error;
mod module;
mod organization_scope;
mod ports;
pub mod use_cases;

pub use dto::{
    BrandCatalogReadModel, CategoryReadModel, ItemReadModel, StoreCatalogReadModel,
    StoreItemListingReadModel,
};
pub use error::ApplicationError;
pub use module::CatalogModule;
pub use organization_scope::{CatalogBrandScope, CatalogStoreScope};
pub use ports::{
    ActiveCatalogContextReadModel, ActiveCatalogQueryService, BrandCatalogQueryService,
    BrandCatalogReadRepository, BrandCatalogRepository, CatalogItemListFilter,
    CategoryQueryService, CategoryReadRepository, CategoryRepository, Clock, IdGenerator,
    ItemQueryService, ItemReadRepository, ItemRepository, OrganizationScopeReader,
    StoreCatalogQueryService, StoreCatalogReadRepository, StoreCatalogRepository,
    StoreItemListingQueryService, StoreItemListingReadRepository, StoreItemListingRepository,
    TransactionContext, TransactionManager,
};
pub use use_cases::{
    AttachStoreCatalog, AttachStoreCatalogInput, BootstrapBrandCatalog, BootstrapBrandCatalogInput,
    BootstrapDefaultCatalog, BootstrapDefaultCatalogInput, BootstrapDefaultCatalogOutcome,
    BootstrapDefaultCategoryInput, BootstrapDefaultItemInput, CreateCategory, CreateCategoryInput,
    CreateItem, CreateItemInput, UpsertStoreItemListing, UpsertStoreItemListingInput,
};
