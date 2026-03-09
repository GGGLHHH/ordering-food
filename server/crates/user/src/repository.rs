use async_trait::async_trait;
use uuid::Uuid;

use ordering_food_shared::error::AppError;

use crate::domain::{NewUser, Phone, UpdateUser, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<User>, AppError>;
    async fn create(&self, new_user: &NewUser) -> Result<User, AppError>;
    async fn update(&self, id: Uuid, update: &UpdateUser) -> Result<Option<User>, AppError>;
}
