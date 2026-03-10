use crate::{ApplicationError, Clock, TransactionManager, UserRepository};
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, UserId, UserIdentity};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindUserIdentityInput {
    pub user_id: String,
    pub identity_type: String,
    pub identifier: String,
}

pub struct BindUserIdentity {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
    clock: Arc<dyn Clock>,
}

impl BindUserIdentity {
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

    pub async fn execute(&self, input: BindUserIdentityInput) -> Result<(), ApplicationError> {
        let mut tx = self.transaction_manager.begin().await?;
        let user_id = UserId::new(input.user_id);
        let mut user = match self.repository.find_by_id(tx.as_mut(), &user_id).await? {
            Some(user) => user,
            None => {
                self.transaction_manager.rollback(tx).await?;
                return Err(ApplicationError::not_found("user was not found"));
            }
        };
        let identity_type = IdentityType::parse(input.identity_type)?;
        let identifier = NormalizedIdentifier::new(input.identifier)?;

        if let Some(existing_user) = self
            .repository
            .find_by_identity(tx.as_mut(), &identity_type, &identifier)
            .await?
            && existing_user.id() != user.id()
        {
            self.transaction_manager.rollback(tx).await?;
            return Err(ApplicationError::conflict(
                "identity is already bound to another user",
            ));
        }

        if let Err(error) = user.bind_identity(
            UserIdentity::new(identity_type, identifier, self.clock.now()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeClock, FakeRepository, FakeTransactionManager};
    use ordering_food_identity_domain::{User, UserProfile};
    use std::sync::Arc;
    use time::macros::datetime;

    #[tokio::test]
    async fn bind_user_identity_updates_existing_aggregate() {
        let repository = Arc::new(FakeRepository::default());
        repository.seed(User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 08:00 UTC),
        ));
        let transactions = Arc::new(FakeTransactionManager::default());
        let use_case = BindUserIdentity::new(
            repository.clone(),
            transactions.clone(),
            Arc::new(FakeClock {
                now: datetime!(2026-03-10 09:00 UTC),
            }),
        );

        use_case
            .execute(BindUserIdentityInput {
                user_id: "user-1".to_string(),
                identity_type: "email".to_string(),
                identifier: "Alice@Example.com".to_string(),
            })
            .await
            .unwrap();

        let users = repository.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.identities().len(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }
}
