use async_trait::async_trait;
use ordering_food_identity_application::{
    AccessTokenClaims, ApplicationError, TokenPair, TokenService,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
    iat: u64,
}

pub struct JwtTokenService {
    secret: String,
    access_ttl_seconds: u64,
    refresh_ttl_seconds: u64,
}

impl JwtTokenService {
    pub fn new(secret: String, access_ttl_seconds: u64, refresh_ttl_seconds: u64) -> Self {
        Self {
            secret,
            access_ttl_seconds,
            refresh_ttl_seconds,
        }
    }
}

#[async_trait]
impl TokenService for JwtTokenService {
    fn generate_token_pair(&self, user_id: &str) -> Result<TokenPair, ApplicationError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ApplicationError::unexpected(format!("system time error: {e}")))?
            .as_secs();

        let access_claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + self.access_ttl_seconds,
        };

        let access_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &access_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| ApplicationError::unexpected(format!("JWT encoding failed: {e}")))?;

        let refresh_token = uuid::Uuid::now_v7().to_string();

        Ok(TokenPair {
            access_token,
            access_token_expires_in: self.access_ttl_seconds,
            refresh_token,
            refresh_token_expires_in: self.refresh_ttl_seconds,
        })
    }

    fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, ApplicationError> {
        let validation = jsonwebtoken::Validation::default();
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|_| ApplicationError::unauthorized("invalid or expired access token"))?;

        Ok(AccessTokenClaims {
            user_id: token_data.claims.sub,
            exp: token_data.claims.exp,
        })
    }
}
