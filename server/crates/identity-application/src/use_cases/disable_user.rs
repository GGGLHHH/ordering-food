use crate::{ApplicationError, Clock, IdentityUnitOfWorkFactory};
use ordering_food_identity_domain::UserId;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableUserInput {
    pub user_id: String,
}

pub struct DisableUser {
    unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
    clock: Arc<dyn Clock>,
}

impl DisableUser {
    pub fn new(
        unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            unit_of_work_factory,
            clock,
        }
    }

    pub async fn execute(&self, input: DisableUserInput) -> Result<(), ApplicationError> {
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

        if user.status() == ordering_food_identity_domain::UserStatus::Disabled {
            unit_of_work.rollback().await?;
            return Err(ApplicationError::conflict("user can no longer be disabled"));
        }

        if let Err(error) = user.disable(self.clock.now()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeClock, FakeIdentityStore, FakeIdentityUnitOfWorkFactory};
    use ordering_food_identity_domain::{User, UserProfile, UserStatus};
    use std::sync::Arc;
    use time::macros::datetime;

    #[tokio::test]
    async fn disable_user_marks_user_as_disabled() {
        let store = Arc::new(FakeIdentityStore::default());
        store.seed_user(User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 08:00 UTC),
        ));
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let use_case = DisableUser::new(
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 09:00 UTC),
            }),
        );

        use_case
            .execute(DisableUserInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();

        let users = store.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.status(), UserStatus::Disabled);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn disable_user_returns_conflict_when_user_is_already_disabled() {
        let store = Arc::new(FakeIdentityStore::default());
        store.seed_user(
            User::rehydrate(
                UserId::new("user-1"),
                UserStatus::Disabled,
                UserProfile::new("Alice", None, None, None).unwrap(),
                Vec::new(),
                datetime!(2026-03-10 08:00 UTC),
                datetime!(2026-03-10 08:30 UTC),
                None,
            )
            .unwrap(),
        );
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let use_case = DisableUser::new(
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 09:00 UTC),
            }),
        );

        let error = use_case
            .execute(DisableUserInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Conflict { ref message }
            if message == "user can no longer be disabled"
        ));
        assert_eq!(*transactions.rollback_count.lock().unwrap(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 0);

        let users = store.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.status(), UserStatus::Disabled);
        assert_eq!(user.updated_at(), datetime!(2026-03-10 08:30 UTC));
    }
}
