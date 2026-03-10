mod dto;
mod error;
mod module;
mod ports;
pub mod use_cases;

#[cfg(test)]
pub(crate) mod testing;

pub use dto::{UserIdentityReadModel, UserProfileReadModel, UserReadModel};
pub use error::ApplicationError;
pub use module::IdentityModule;
pub use ports::{
    Clock, IdGenerator, TransactionContext, TransactionManager, UserQueryService,
    UserReadRepository, UserRepository,
};
pub use use_cases::{
    BindUserIdentity, BindUserIdentityInput, CreateUser, CreateUserIdentityInput, CreateUserInput,
    DisableUser, DisableUserInput, SoftDeleteUser, SoftDeleteUserInput, UpdateUserProfile,
    UpdateUserProfileInput,
};
