use crate::{ApplicationError, Clock, IdentityUnitOfWorkFactory};
use ordering_food_identity_domain::{IdentityType, NormalizedIdentifier, UserId, UserIdentity};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindUserIdentityInput {
    pub user_id: String,
    pub identity_type: String,
    pub identifier: String,
}

pub struct BindUserIdentity {
    unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
    clock: Arc<dyn Clock>,
}

impl BindUserIdentity {
    pub fn new(
        unit_of_work_factory: Arc<dyn IdentityUnitOfWorkFactory>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            unit_of_work_factory,
            clock,
        }
    }

    pub async fn execute(&self, input: BindUserIdentityInput) -> Result<(), ApplicationError> {
        let identity_type = IdentityType::parse(input.identity_type)?;
        let identifier = NormalizedIdentifier::new(input.identifier)?;
        let now = self.clock.now();
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

        if let Some(existing_user) = match unit_of_work
            .find_user_by_identity(&identity_type, &identifier)
            .await
        {
            Ok(existing_user) => existing_user,
            Err(error) => {
                unit_of_work.rollback().await?;
                return Err(error);
            }
        } && existing_user.id() != user.id()
        {
            unit_of_work.rollback().await?;
            return Err(ApplicationError::conflict(
                "identity is already bound to another user",
            ));
        }

        if let Err(error) =
            user.bind_identity(UserIdentity::new(identity_type, identifier, now), now)
        {
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
    use ordering_food_identity_domain::{User, UserProfile};
    use std::sync::Arc;
    use time::macros::datetime;

    #[tokio::test]
    async fn bind_user_identity_updates_existing_aggregate() {
        let store = Arc::new(FakeIdentityStore::default());
        store.seed_user(User::create(
            UserId::new("user-1"),
            UserProfile::new("Alice", None, None, None).unwrap(),
            datetime!(2026-03-10 08:00 UTC),
        ));
        let transactions = Arc::new(FakeIdentityUnitOfWorkFactory::new(store.clone()));
        let use_case = BindUserIdentity::new(
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

        let users = store.users();
        let user = users.get("user-1").unwrap();
        assert_eq!(user.identities().len(), 1);
        assert_eq!(*transactions.commit_count.lock().unwrap(), 1);
    }
}
