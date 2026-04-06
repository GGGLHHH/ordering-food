use async_trait::async_trait;
use ordering_food_identity_application::{
    IdGenerator, IdentityModule, TokenService, UserQueryService,
};
use ordering_food_identity_domain::UserId;
use ordering_food_identity_infrastructure_auth::{
    Argon2PasswordHasher, JwtTokenService, RedisRefreshTokenStore,
};
use ordering_food_identity_infrastructure_sqlx::build_identity_module;
use ordering_food_identity_published::{
    AccessTokenVerifier, IdentityCollaborationError, SubjectLookupGateway, SubjectRef,
    SubjectStatus,
};
use ordering_food_platform_kernel::Clock;
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityContextConfig {
    jwt_secret: String,
    access_token_ttl_seconds: u64,
    refresh_token_ttl_seconds: u64,
}

impl IdentityContextConfig {
    pub fn new(
        jwt_secret: impl Into<String>,
        access_token_ttl_seconds: u64,
        refresh_token_ttl_seconds: u64,
    ) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            access_token_ttl_seconds,
            refresh_token_ttl_seconds,
        }
    }
}

#[derive(Clone)]
pub struct IdentityContextRuntime {
    pub module: Arc<IdentityModule>,
    pub access_token_verifier: Arc<dyn AccessTokenVerifier>,
    pub subject_lookup_gateway: Arc<dyn SubjectLookupGateway>,
}

pub fn build_identity_context_runtime(
    pg_pool: PgPool,
    clock: Arc<dyn Clock>,
    redis_client: RedisClient,
    config: IdentityContextConfig,
) -> IdentityContextRuntime {
    let password_hasher = Arc::new(Argon2PasswordHasher);
    let id_generator = Arc::new(UuidV4UserIdGenerator);
    let jwt_token_service = Arc::new(JwtTokenService::new(
        config.jwt_secret,
        config.access_token_ttl_seconds,
        config.refresh_token_ttl_seconds,
    ));
    let token_service: Arc<dyn TokenService> = jwt_token_service.clone();
    let access_token_verifier: Arc<dyn AccessTokenVerifier> = jwt_token_service;
    let refresh_token_store = Arc::new(RedisRefreshTokenStore::new(redis_client));
    let module = build_identity_module(
        pg_pool,
        clock,
        id_generator,
        password_hasher,
        token_service,
        refresh_token_store,
    );
    let subject_lookup_gateway = build_subject_lookup_gateway(module.user_queries().clone());

    IdentityContextRuntime {
        module,
        access_token_verifier,
        subject_lookup_gateway,
    }
}

fn build_subject_lookup_gateway(
    user_queries: Arc<UserQueryService>,
) -> Arc<dyn SubjectLookupGateway> {
    Arc::new(SqlxSubjectLookupGateway::new(user_queries))
}

struct SqlxSubjectLookupGateway {
    user_queries: Arc<UserQueryService>,
}

impl SqlxSubjectLookupGateway {
    fn new(user_queries: Arc<UserQueryService>) -> Self {
        Self { user_queries }
    }
}

#[async_trait]
impl SubjectLookupGateway for SqlxSubjectLookupGateway {
    async fn get_by_id(
        &self,
        subject_id: &str,
    ) -> Result<Option<SubjectRef>, IdentityCollaborationError> {
        let read_model = self
            .user_queries
            .get_by_id(subject_id)
            .await
            .map_err(|error| IdentityCollaborationError::new(error.to_string()))?;

        Ok(read_model.map(|user| SubjectRef::new(user.user_id, map_subject_status(&user.status))))
    }
}

fn map_subject_status(status: &str) -> SubjectStatus {
    if status.eq_ignore_ascii_case("active") {
        SubjectStatus::Active
    } else {
        SubjectStatus::Disabled
    }
}

struct UuidV4UserIdGenerator;

impl IdGenerator for UuidV4UserIdGenerator {
    fn next_user_id(&self) -> UserId {
        UserId::new(Uuid::new_v4().to_string())
    }
}
