mod dto;
mod error;
mod module;
mod ports;
pub mod use_cases;

#[cfg(test)]
pub(crate) mod testing;

pub use dto::{
    AccessTokenClaims, StoredCredential, TokenPair, UserIdentityReadModel, UserProfileReadModel,
    UserReadModel,
};
pub use error::ApplicationError;
pub use module::IdentityModule;
pub use ports::{
    Clock, IdGenerator, IdentityUnitOfWork, IdentityUnitOfWorkFactory, PasswordHasher,
    RefreshTokenStore, TokenService, UserQueryService, UserReadRepository,
};
pub use use_cases::{
    BindUserIdentity, BindUserIdentityInput, CreateUser, CreateUserIdentityInput, CreateUserInput,
    DisableUser, DisableUserInput, Login, LoginInput, LoginOutput, Logout, LogoutInput,
    RefreshToken, RefreshTokenInput, RefreshTokenOutput, SoftDeleteUser, SoftDeleteUserInput,
    UpdateUserProfile, UpdateUserProfileInput,
};
