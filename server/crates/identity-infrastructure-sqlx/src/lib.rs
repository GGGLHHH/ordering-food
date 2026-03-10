mod module;
mod transaction;
mod user_read_repository;
mod user_repository;

pub use module::build_identity_module;
pub use transaction::SqlxTransactionManager;
pub use user_read_repository::SqlxUserReadRepository;
pub use user_repository::SqlxUserRepository;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
