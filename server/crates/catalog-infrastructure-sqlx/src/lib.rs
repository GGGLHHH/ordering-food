//! SQLx persistence for the Catalog bounded context.

mod brand_catalog_read_repository;
mod brand_catalog_repository;
mod category_read_repository;
mod category_repository;
mod item_read_repository;
mod item_repository;
mod module;
mod store_catalog_read_repository;
mod store_catalog_repository;
mod store_item_listing_repository;
mod transaction;

use ordering_food_catalog_application::ApplicationError;
use ordering_food_catalog_domain::{DisplayRule, SellableStatus};
use uuid::Uuid;

pub use brand_catalog_read_repository::SqlxBrandCatalogReadRepository;
pub use brand_catalog_repository::SqlxBrandCatalogRepository;
pub use category_read_repository::SqlxCategoryReadRepository;
pub use category_repository::SqlxCategoryRepository;
pub use item_read_repository::SqlxItemReadRepository;
pub use item_repository::SqlxItemRepository;
pub use module::{CatalogSqlxModule, build_catalog_sqlx_module};
pub use store_catalog_read_repository::SqlxStoreCatalogReadRepository;
pub use store_catalog_repository::SqlxStoreCatalogRepository;
pub use store_item_listing_repository::SqlxStoreItemListingRepository;
pub use transaction::SqlxTransactionManager;

fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, ApplicationError> {
    Uuid::parse_str(value)
        .map_err(|_| ApplicationError::validation(format!("{field} must be a valid UUID")))
}

fn parse_sellable_status(value: &str) -> Result<SellableStatus, ApplicationError> {
    match value {
        "sellable" => Ok(SellableStatus::Sellable),
        "unsellable" => Ok(SellableStatus::Unsellable),
        _ => Err(ApplicationError::unexpected(format!(
            "unsupported catalog sellable status `{value}`"
        ))),
    }
}

fn sellable_status_as_str(status: SellableStatus) -> &'static str {
    status.as_str()
}

fn parse_display_rule(value: &str) -> Result<DisplayRule, ApplicationError> {
    match value {
        "listed" => Ok(DisplayRule::listed()),
        "hidden" => Ok(DisplayRule::hidden()),
        _ => Err(ApplicationError::unexpected(format!(
            "unsupported catalog display rule `{value}`"
        ))),
    }
}

fn display_rule_as_str(display_rule: DisplayRule) -> &'static str {
    if display_rule.is_hidden() {
        "hidden"
    } else {
        "listed"
    }
}
