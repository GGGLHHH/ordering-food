//! SQLx persistence for the Access bounded context.

mod db_roles;
mod repository;

pub use repository::SqlxAccessGrantRepository;
