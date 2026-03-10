use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_identity_application::{ApplicationError, TransactionContext, UserRepository};
use ordering_food_identity_domain::{
    IdentityType, NormalizedIdentifier, User, UserId, UserIdentity, UserProfile, UserStatus,
};
use ordering_food_shared_kernel::Identifier;
use sqlx::{Postgres, Row, Transaction};

const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";
const USER_IDENTITIES_IDENTIFIER_UNIQUE_CONSTRAINT: &str = "user_identities_identifier_unique";
const IDENTITY_CONFLICT_MESSAGE: &str = "identity is already bound to another user";

#[derive(Debug, Default)]
pub struct SqlxUserRepository;

impl SqlxUserRepository {
    fn transaction(
        tx: &mut dyn TransactionContext,
    ) -> Result<&mut Transaction<'static, Postgres>, ApplicationError> {
        tx.as_any_mut()
            .downcast_mut::<SqlxTransactionContext>()
            .map(SqlxTransactionContext::transaction_mut)
            .ok_or_else(|| {
                ApplicationError::unexpected("unexpected transaction context implementation")
            })
    }

    fn map_identity_write_error(message: &'static str, error: sqlx::Error) -> ApplicationError {
        if Self::is_identity_uniqueness_violation(&error) {
            ApplicationError::conflict(IDENTITY_CONFLICT_MESSAGE)
        } else {
            ApplicationError::unexpected_with_source(message, error)
        }
    }

    fn is_identity_uniqueness_violation(error: &sqlx::Error) -> bool {
        error.as_database_error().is_some_and(|database_error| {
            Self::matches_identity_uniqueness_violation(
                database_error.code().as_deref(),
                database_error.constraint(),
            )
        })
    }

    fn matches_identity_uniqueness_violation(code: Option<&str>, constraint: Option<&str>) -> bool {
        code == Some(UNIQUE_VIOLATION_SQLSTATE)
            && constraint == Some(USER_IDENTITIES_IDENTIFIER_UNIQUE_CONSTRAINT)
    }

    async fn load_user(
        transaction: &mut Transaction<'static, Postgres>,
        user_id: &UserId,
    ) -> Result<Option<User>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT
                u.id,
                u.status,
                u.created_at,
                u.updated_at,
                u.deleted_at,
                p.display_name,
                p.given_name,
                p.family_name,
                p.avatar_url
            FROM identity.users u
            INNER JOIN identity.user_profiles p ON p.user_id = u.id
            WHERE u.id = $1
            "#,
        )
        .bind(user_id.as_str())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load identity user aggregate",
                error,
            )
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        let identities = sqlx::query(
            r#"
            SELECT identity_type, identifier_normalized, bound_at
            FROM identity.user_identities
            WHERE user_id = $1
            ORDER BY identity_type, identifier_normalized
            "#,
        )
        .bind(user_id.as_str())
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to load identity user identities",
                error,
            )
        })?
        .into_iter()
        .map(|identity| {
            Ok(UserIdentity::new(
                IdentityType::parse(identity.get::<String, _>("identity_type"))?,
                NormalizedIdentifier::new(identity.get::<String, _>("identifier_normalized"))?,
                identity.get("bound_at"),
            ))
        })
        .collect::<Result<Vec<_>, ordering_food_identity_domain::DomainError>>()?;

        let user = User::rehydrate(
            UserId::new(row.get::<String, _>("id")),
            UserStatus::parse(row.get::<String, _>("status"))?,
            UserProfile::new(
                row.get::<String, _>("display_name"),
                row.get::<Option<String>, _>("given_name"),
                row.get::<Option<String>, _>("family_name"),
                row.get::<Option<String>, _>("avatar_url"),
            )?,
            identities,
            row.get("created_at"),
            row.get("updated_at"),
            row.get("deleted_at"),
        )?;

        Ok(Some(user))
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        user_id: &UserId,
    ) -> Result<Option<User>, ApplicationError> {
        Self::load_user(Self::transaction(tx)?, user_id).await
    }

    async fn find_by_identity(
        &self,
        tx: &mut dyn TransactionContext,
        identity_type: &IdentityType,
        identifier: &NormalizedIdentifier,
    ) -> Result<Option<User>, ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let user_id = sqlx::query(
            r#"
            SELECT user_id
            FROM identity.user_identities
            WHERE identity_type = $1 AND identifier_normalized = $2
            "#,
        )
        .bind(identity_type.as_str())
        .bind(identifier.as_str())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to lookup identity by normalized identifier",
                error,
            )
        })?
        .map(|row| UserId::new(row.get::<String, _>("user_id")));

        match user_id {
            Some(user_id) => Self::load_user(transaction, &user_id).await,
            None => Ok(None),
        }
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        user: &User,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;

        sqlx::query(
            r#"
            INSERT INTO identity.users (id, status, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user.id().as_str())
        .bind(user.status().as_str())
        .bind(user.created_at())
        .bind(user.updated_at())
        .bind(user.deleted_at())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to insert identity user", error)
        })?;

        sqlx::query(
            r#"
            INSERT INTO identity.user_profiles (
                user_id,
                display_name,
                given_name,
                family_name,
                avatar_url
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user.id().as_str())
        .bind(user.profile().display_name())
        .bind(user.profile().given_name())
        .bind(user.profile().family_name())
        .bind(user.profile().avatar_url())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to insert identity user profile",
                error,
            )
        })?;

        for identity in user.identities() {
            sqlx::query(
                r#"
                INSERT INTO identity.user_identities (
                    user_id,
                    identity_type,
                    identifier_normalized,
                    bound_at
                )
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(user.id().as_str())
            .bind(identity.identity_type().as_str())
            .bind(identity.identifier_normalized().as_str())
            .bind(identity.bound_at())
            .execute(&mut **transaction)
            .await
            .map_err(|error| {
                Self::map_identity_write_error("failed to insert identity user identity", error)
            })?;
        }

        Ok(())
    }

    async fn update(
        &self,
        tx: &mut dyn TransactionContext,
        user: &User,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;

        let result = sqlx::query(
            r#"
            UPDATE identity.users
            SET status = $2, updated_at = $3, deleted_at = $4
            WHERE id = $1
            "#,
        )
        .bind(user.id().as_str())
        .bind(user.status().as_str())
        .bind(user.updated_at())
        .bind(user.deleted_at())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to update identity user", error)
        })?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::not_found("user was not found"));
        }

        sqlx::query(
            r#"
            UPDATE identity.user_profiles
            SET display_name = $2, given_name = $3, family_name = $4, avatar_url = $5
            WHERE user_id = $1
            "#,
        )
        .bind(user.id().as_str())
        .bind(user.profile().display_name())
        .bind(user.profile().given_name())
        .bind(user.profile().family_name())
        .bind(user.profile().avatar_url())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to update identity user profile",
                error,
            )
        })?;

        sqlx::query("DELETE FROM identity.user_identities WHERE user_id = $1")
            .bind(user.id().as_str())
            .execute(&mut **transaction)
            .await
            .map_err(|error| {
                ApplicationError::unexpected_with_source(
                    "failed to replace identity user identities",
                    error,
                )
            })?;

        for identity in user.identities() {
            sqlx::query(
                r#"
                INSERT INTO identity.user_identities (
                    user_id,
                    identity_type,
                    identifier_normalized,
                    bound_at
                )
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(user.id().as_str())
            .bind(identity.identity_type().as_str())
            .bind(identity.identifier_normalized().as_str())
            .bind(identity.bound_at())
            .execute(&mut **transaction)
            .await
            .map_err(|error| {
                Self::map_identity_write_error("failed to insert identity user identity", error)
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SqlxUserRepository;

    #[test]
    fn matches_identity_unique_violation_for_target_constraint() {
        assert!(SqlxUserRepository::matches_identity_uniqueness_violation(
            Some("23505"),
            Some("user_identities_identifier_unique"),
        ));
    }

    #[test]
    fn ignores_other_unique_constraints_or_error_codes() {
        assert!(!SqlxUserRepository::matches_identity_uniqueness_violation(
            Some("23505"),
            Some("other_constraint"),
        ));
        assert!(!SqlxUserRepository::matches_identity_uniqueness_violation(
            Some("23503"),
            Some("user_identities_identifier_unique"),
        ));
        assert!(!SqlxUserRepository::matches_identity_uniqueness_violation(
            None,
            Some("user_identities_identifier_unique"),
        ));
    }
}
