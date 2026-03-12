use async_trait::async_trait;
use ordering_food_identity_application::{ApplicationError, PasswordHasher};

pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, raw_password: &str) -> Result<String, ApplicationError> {
        let password = raw_password.to_string();
        tokio::task::spawn_blocking(move || {
            use argon2::{
                Argon2,
                password_hash::{PasswordHasher as _, SaltString, rand_core::OsRng},
            };
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| ApplicationError::unexpected(format!("password hashing failed: {e}")))
        })
        .await
        .map_err(|e| ApplicationError::unexpected(format!("password hash task failed: {e}")))?
    }

    async fn verify(&self, raw_password: &str, hash: &str) -> Result<bool, ApplicationError> {
        let password = raw_password.to_string();
        let hash = hash.to_string();
        tokio::task::spawn_blocking(move || {
            use argon2::{
                Argon2,
                password_hash::{PasswordHash, PasswordVerifier},
            };
            let parsed = PasswordHash::new(&hash)
                .map_err(|e| ApplicationError::unexpected(format!("invalid password hash: {e}")))?;
            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok())
        })
        .await
        .map_err(|e| ApplicationError::unexpected(format!("password verify task failed: {e}")))?
    }
}
