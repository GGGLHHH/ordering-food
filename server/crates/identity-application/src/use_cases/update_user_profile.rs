use crate::{ApplicationError, Clock, TransactionManager, UserRepository};
use ordering_food_identity_domain::{UserId, UserProfile};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateUserProfileInput {
    pub user_id: String,
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub avatar_url: Option<String>,
}

pub struct UpdateUserProfile {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
}

impl UpdateUserProfile {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            repository,
            transaction_manager,
            clock,
        }
    }

    pub async fn execute(&self, input: UpdateUserProfileInput) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let user_id = UserId::new(input.user_id);
        let mut user = match self.repository.find_by_id(tx.as_mut(), &user_id).await? {
            Some(user) => user,
            None => {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::not_found("user was not found"));
            }
        };

        if let Err(error) = user.update_profile(
            UserProfile::new(
                input.display_name,
                input.given_name,
                input.family_name,
                input.avatar_url,
            )?,
            self.clock.now(),
        ) {
            self.transaction_manager.rollback(tx).await?;
            return Err(error.into());
        }

        if let Err(error) = self.repository.update(tx.as_mut(), &user).await {
            self.transaction_manager.rollback(tx).await?;
            return Err(error);
        }

        self.transaction_manager.commit(tx).await
    }
}
