use std::sync::Arc;

use uuid::Uuid;

use ordering_food_shared::error::AppError;

use crate::domain::{NewUser, Phone, UpdateUser, User, UserDomainError};
use crate::repository::UserRepository;

pub struct UserService {
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    /// Login/register by phone number. Creates user if not exists.
    pub async fn find_or_create_by_phone(&self, phone_raw: &str) -> Result<User, AppError> {
        let phone = Phone::new(phone_raw).map_err(|e| match e {
            UserDomainError::InvalidPhone(msg) => AppError::validation_error(msg),
        })?;

        if let Some(user) = self.repo.find_by_phone(&phone).await? {
            return Ok(user);
        }

        let new_user = NewUser { phone };
        self.repo.create(&new_user).await
    }

    /// Get user by ID.
    pub async fn get_by_id(&self, id: Uuid) -> Result<User, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))
    }

    /// Update user profile.
    pub async fn update_profile(&self, id: Uuid, update: UpdateUser) -> Result<User, AppError> {
        self.repo
            .update(id, &update)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))
    }
}
