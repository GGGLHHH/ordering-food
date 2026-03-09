use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use ordering_food_shared::error::AppError;

use crate::domain::{NewUser, Phone, Role, UpdateUser, User, UserStatus};
use crate::repository::UserRepository;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Intermediate row struct isolating SQLx types from domain types.
#[derive(FromRow)]
struct UserRow {
    id: i64,
    phone: String,
    nickname: String,
    avatar_url: String,
    role: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserRow {
    fn into_domain(self) -> User {
        User {
            id: self.id,
            phone: self.phone,
            nickname: self.nickname,
            avatar_url: self.avatar_url,
            role: Role::from_str_value(&self.role).unwrap_or(Role::Customer),
            status: UserStatus::from_str_value(&self.status).unwrap_or(UserStatus::Active),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, phone, nickname, avatar_url, role, status, created_at, updated_at \
             FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal_with_source("failed to query user by id", e))?;

        Ok(row.map(UserRow::into_domain))
    }

    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, phone, nickname, avatar_url, role, status, created_at, updated_at \
             FROM users WHERE phone = $1",
        )
        .bind(phone.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal_with_source("failed to query user by phone", e))?;

        Ok(row.map(UserRow::into_domain))
    }

    async fn create(&self, new_user: &NewUser) -> Result<User, AppError> {
        // INSERT ... ON CONFLICT DO NOTHING handles concurrent registration race.
        sqlx::query("INSERT INTO users (phone) VALUES ($1) ON CONFLICT (phone) DO NOTHING")
            .bind(new_user.phone.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal_with_source("failed to insert user", e))?;

        // Re-query to get the full row (whether just inserted or already existed).
        self.find_by_phone(&new_user.phone)
            .await?
            .ok_or_else(|| AppError::internal("user not found after insert"))
    }

    async fn update(&self, id: i64, update: &UpdateUser) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "UPDATE users SET \
                nickname   = COALESCE($2, nickname), \
                avatar_url = COALESCE($3, avatar_url), \
                role       = COALESCE($4, role), \
                status     = COALESCE($5, status), \
                updated_at = NOW() \
            WHERE id = $1 \
            RETURNING id, phone, nickname, avatar_url, role, status, created_at, updated_at",
        )
        .bind(id)
        .bind(update.nickname.as_deref())
        .bind(update.avatar_url.as_deref())
        .bind(update.role.map(|r| r.as_str()))
        .bind(update.status.map(|s| s.as_str()))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            // Detect unique constraint violation (PostgreSQL error code 23505).
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23505")
            {
                return AppError::validation_error("phone number already in use");
            }
            AppError::internal_with_source("failed to update user", e)
        })?;

        Ok(row.map(UserRow::into_domain))
    }
}
