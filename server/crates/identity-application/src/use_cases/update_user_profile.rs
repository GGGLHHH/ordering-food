use crate::{ApplicationError, Clock, IdentityUnitOfWorkFactory};
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
    unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
    clock: Arc<dyn Clock>,
}

impl UpdateUserProfile {
    pub fn new(
        unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            unit_of_work_factory,
            clock,
        }
    }

    pub async fn execute(&self, input: UpdateUserProfileInput) -> Result<(), ApplicationError> {
        let profile = UserProfile::new(
            input.display_name,
            input.given_name,
            input.family_name,
            input.avatar_url,
        )?;
        let mut unit_of_work = self.unit_of_work_factory.begin().await?;
        let user_id = UserId::new(input.user_id);
        let mut user = match unit_of_work.find_user_by_id(&user_id).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                unit_of_work.rollback().await?;
                return Err(ApplicationError::not_found("user was not found"));
            }
            Err(error) => {
                unit_of_work.rollback().await?;
                return Err(error);
            }
        };

        if let Err(error) = user.update_profile(profile, self.clock.now()) {
            unit_of_work.rollback().await?;
            return Err(error.into());
        }

        if let Err(error) = unit_of_work.update_user(&user).await {
            unit_of_work.rollback().await?;
            return Err(error);
        }

        unit_of_work.commit().await
    }
}
