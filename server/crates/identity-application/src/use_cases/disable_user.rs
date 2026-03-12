use crate::{ApplicationError, Clock, TransactionManager, UserRepository};
use ordering_food_identity_domain::UserId;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableUserInput {
    pub user_id: String,
}

pub struct DisableUser {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
}

impl DisableUser {
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

    pub async fn execute(&self, input: DisableUserInput) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let user_id = UserId::new(input.user_id);
        let mut user = match self.repository.find_by_id(tx.as_mut(), &user_id).await? {
            Some(user) => user,
            None => {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::not_found("user was not found"));
            }
        };

        if user.status() == ordering_food_identity_domain::UserStatus::Disabled {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::conflict("user can no longer be disabled"));
        }

        if let Err(error) = user.disable(self.clock.now()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeClock, FakeRepository, FakeTransactionManager};
    use ordering_food_identity_domain::{User, UserProfile, UserStatus};
    use std::sync::Arc;
    use time::macros::datetime;

    #[tokio::test]
    async fn disable_user_marks_user_as_disabled() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 08:00 UTC),
        ));
        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = DisableUser::new(
            repository.clone(),
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

        let users = repository.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.status(), UserStatus::Disabled);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn disable_user_returns_conflict_when_user_is_already_disabled() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(
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
        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = DisableUser::new(
            repository.clone(),
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

        let users = repository.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.status(), UserStatus::Disabled);
        assert_eq!(user.updated_at(), datetime!(2026-03-10 08:30 UTC));
    }
}
