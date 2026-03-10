use crate::{ApplicationError, Clock, TransactionManager, UserRepository};
use ordering_food_identity_domain::UserId;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftDeleteUserInput {
    pub user_id: String,
}

pub struct SoftDeleteUser {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
}

impl SoftDeleteUser {
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

    pub async fn execute(&self, input: SoftDeleteUserInput) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let user_id = UserId::new(input.user_id);
        let mut user = match self.repository.find_by_id(tx.as_mut(), &user_id).await? {
            Some(user) => user,
            None => {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::not_found("user was not found"));
            }
        };

        if let Err(error) = user.soft_delete(self.clock.now()) {
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
    async fn soft_delete_user_sets_deleted_at_and_disables_user() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 08:00 UTC),
        ));
        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = SoftDeleteUser::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 09:00 UTC),
            }),
        );

        use_case
            .execute(SoftDeleteUserInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();

        let users = repository.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.status(), UserStatus::Disabled);
        assert!(user.deleted_at().is_some());
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }
}
