use async_trait::async_trait;
use ordering_food_identity_application::{
    ApplicationError, UserIdentityReadModel, UserProfileReadModel, UserReadModel,
    UserReadRepository,
};
use ordering_food_identity_domain::UserId;
use ordering_food_shared_kernel::Identifier;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxUserReadRepository {
    pool: PgPool,
}

impl SqlxUserReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_user_id(user_id: &UserId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(user_id.as_str())
            .map_err(|_| ApplicationError::validation("user id must be a valid UUID"))
    }
}

#[async_trait]
impl UserReadRepository for SqlxUserReadRepository {
    async fn get_by_id(&self, user_id: &UserId) -> Result<Option<UserReadModel>, ApplicationError> {
        let user_id = Self::parse_user_id(user_id)?;
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
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query identity read model", error)
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
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query identity read model identities",
                error,
            )
        })?
        .into_iter()
        .map(|identity| UserIdentityReadModel {
            identity_type: identity.get("identity_type"),
            identifier_normalized: identity.get("identifier_normalized"),
            bound_at: identity.get("bound_at"),
        })
        .collect();

        Ok(Some(UserReadModel {
            user_id: row.get::<Uuid, _>("id").to_string(),
            status: row.get("status"),
            profile: UserProfileReadModel {
                display_name: row.get("display_name"),
                given_name: row.get("given_name"),
                family_name: row.get("family_name"),
                avatar_url: row.get("avatar_url"),
            },
            identities,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }
}
