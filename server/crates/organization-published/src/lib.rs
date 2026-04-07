//! Published contracts for the Organization bounded context.

mod events;
mod refs;
mod store_gateway;
mod store_scope;

pub use events::StoreStatusChanged;
pub use refs::{BrandRef, StoreRef};
pub use store_gateway::{BrandLookupGateway, OrganizationCollaborationError, StoreScopeGateway};
pub use store_scope::StoreSummary;
