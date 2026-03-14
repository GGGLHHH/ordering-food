use ordering_food_database_infrastructure_sqlx::MIGRATOR;
use ordering_food_identity_application::{
    ApplicationError, CredentialRepository, TransactionManager, UserReadRepository, UserRepository,
};
use ordering_food_identity_domain::{
    IdentityType, NormalizedIdentifier, User, UserId, UserIdentity, UserProfile, UserStatus,
};
use ordering_food_identity_infrastructure_sqlx::{
    SqlxCredentialRepository, SqlxTransactionManager, SqlxUserReadRepository, SqlxUserRepository,
};
use ordering_food_shared_kernel::{Identifier, Timestamp};
use sqlx::{PgPool, Row, types::time::OffsetDateTime};
use uuid::Uuid;

fn unique_uuid() -> Uuid {
    Uuid::now_v7()
}

fn unique_email(prefix: &str) -> String {
    format!("{prefix}-{}@example.com", unique_uuid())
}

fn fixed_timestamp(seconds: i64) -> Timestamp {
    OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}

fn make_user(user_id: Uuid, display_name: &str, email: &str, created_at: Timestamp) -> User {
    let mut user = User::create(
        UserId::new(user_id.to_string()),
        UserProfile::new(display_name, None, None, None).unwrap(),
        created_at,
    );
    user.bind_identity(
        UserIdentity::new(
            IdentityType::Email,
            NormalizedIdentifier::new(email).unwrap(),
            created_at,
        ),
        created_at,
    )
    .unwrap();
    user
}

async fn insert_user(pool: &PgPool, user: &User) {
    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), user).await.unwrap();
    transactions.commit(tx).await.unwrap();
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_repository_inserts_and_loads_uuid_user(pool: PgPool) {
    let user_id = unique_uuid();
    let email = unique_email("repo-insert-load");
    let user = make_user(user_id, "Alice", &email, fixed_timestamp(1_700_000_000));

    insert_user(&pool, &user).await;

    let users_count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity.users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let profiles_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity.user_profiles WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let identities_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity.user_identities WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(users_count, 1);
    assert_eq!(profiles_count, 1);
    assert_eq!(identities_count, 1);

    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    let loaded = repository
        .find_by_id(tx.as_mut(), user.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.id().as_str(), user.id().as_str());
    assert_eq!(loaded.profile().display_name(), "Alice");
    assert_eq!(loaded.identities().len(), 1);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_repository_finds_by_identity(pool: PgPool) {
    let user_id = unique_uuid();
    let email = unique_email("repo-find-identity");
    let user = make_user(user_id, "Alice", &email, fixed_timestamp(1_700_000_100));
    insert_user(&pool, &user).await;

    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    let loaded = repository
        .find_by_identity(
            tx.as_mut(),
            &IdentityType::Email,
            &NormalizedIdentifier::new(email).unwrap(),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.id().as_str(), user.id().as_str());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_repository_returns_none_for_missing_identity(pool: PgPool) {
    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let mut tx = transactions.begin().await.unwrap();

    let loaded = repository
        .find_by_identity(
            tx.as_mut(),
            &IdentityType::Email,
            &NormalizedIdentifier::new("missing@example.com").unwrap(),
        )
        .await
        .unwrap();

    assert!(loaded.is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_repository_updates_profile_status_and_identities(pool: PgPool) {
    let created_at = fixed_timestamp(1_700_000_200);
    let user_id = unique_uuid();
    let email = unique_email("repo-update");
    let phone_identity = format!("phone-{}", unique_uuid());
    let mut user = make_user(user_id, "Alice", &email, created_at);
    insert_user(&pool, &user).await;

    user.update_profile(
        UserProfile::new("Alice Updated", Some("Alice".to_string()), None, None).unwrap(),
        fixed_timestamp(1_700_000_300),
    )
    .unwrap();
    user.bind_identity(
        UserIdentity::new(
            IdentityType::Phone,
            NormalizedIdentifier::new(&phone_identity).unwrap(),
            fixed_timestamp(1_700_000_300),
        ),
        fixed_timestamp(1_700_000_300),
    )
    .unwrap();
    user.disable(fixed_timestamp(1_700_000_400)).unwrap();

    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.update(tx.as_mut(), &user).await.unwrap();
    transactions.commit(tx).await.unwrap();

    let mut tx = transactions.begin().await.unwrap();
    let loaded = repository
        .find_by_id(tx.as_mut(), user.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.status(), UserStatus::Disabled);
    assert_eq!(loaded.profile().display_name(), "Alice Updated");
    assert_eq!(loaded.identities().len(), 2);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_repository_update_returns_not_found_for_missing_user(pool: PgPool) {
    let missing_user = make_user(
        unique_uuid(),
        "Missing",
        &unique_email("repo-missing-user"),
        fixed_timestamp(1_700_000_250),
    );
    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let mut tx = transactions.begin().await.unwrap();

    let error = repository
        .update(tx.as_mut(), &missing_user)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::NotFound { ref message } if message == "user was not found")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_repository_rejects_invalid_uuid_string(pool: PgPool) {
    let repository = SqlxUserRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let mut tx = transactions.begin().await.unwrap();
    let error = repository
        .find_by_id(tx.as_mut(), &UserId::new("not-a-uuid"))
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::Validation { ref message } if message == "user id must be a valid UUID")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_read_repository_returns_none_for_missing_user(pool: PgPool) {
    let read_repository = SqlxUserReadRepository::new(pool);

    let read_model = read_repository
        .get_by_id(&UserId::new(unique_uuid().to_string()))
        .await
        .unwrap();

    assert!(read_model.is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_user_read_repository_returns_joined_read_model(pool: PgPool) {
    let user_id = unique_uuid();
    let email = unique_email("repo-read-model");
    let user = make_user(user_id, "Alice", &email, fixed_timestamp(1_700_000_500));
    insert_user(&pool, &user).await;

    let read_repository = SqlxUserReadRepository::new(pool);
    let read_model = read_repository.get_by_id(user.id()).await.unwrap().unwrap();

    assert_eq!(read_model.user_id, user.id().as_str());
    assert_eq!(read_model.profile.display_name, "Alice");
    assert_eq!(read_model.identities.len(), 1);
    assert_eq!(read_model.identities[0].identifier_normalized, email);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_credential_repository_returns_none_for_missing_credential(pool: PgPool) {
    let user_id = unique_uuid();
    let email = unique_email("repo-credential-missing");
    let user = make_user(user_id, "Alice", &email, fixed_timestamp(1_700_000_550));
    insert_user(&pool, &user).await;

    let repository = SqlxCredentialRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let mut tx = transactions.begin().await.unwrap();
    let credential = repository
        .find_by_user_id(tx.as_mut(), user.id())
        .await
        .unwrap();

    assert!(credential.is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_credential_repository_upsert_and_find_by_user_id(pool: PgPool) {
    let user_id = unique_uuid();
    let email = unique_email("repo-credential-upsert");
    let user = make_user(user_id, "Alice", &email, fixed_timestamp(1_700_000_600));
    insert_user(&pool, &user).await;

    let repository = SqlxCredentialRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository
        .upsert(
            tx.as_mut(),
            user.id(),
            "hashed:secret123",
            fixed_timestamp(1_700_000_600),
        )
        .await
        .unwrap();
    transactions.commit(tx).await.unwrap();

    let mut tx = transactions.begin().await.unwrap();
    let credential = repository
        .find_by_user_id(tx.as_mut(), user.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(credential.user_id, user.id().as_str());
    assert_eq!(credential.password_hash, "hashed:secret123");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_credential_repository_upsert_updates_hash_and_timestamp(pool: PgPool) {
    let user_id = unique_uuid();
    let email = unique_email("repo-credential-update");
    let user = make_user(user_id, "Alice", &email, fixed_timestamp(1_700_000_700));
    insert_user(&pool, &user).await;

    let repository = SqlxCredentialRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());

    let mut tx = transactions.begin().await.unwrap();
    repository
        .upsert(
            tx.as_mut(),
            user.id(),
            "hashed:first",
            fixed_timestamp(1_700_000_700),
        )
        .await
        .unwrap();
    transactions.commit(tx).await.unwrap();

    let mut tx = transactions.begin().await.unwrap();
    repository
        .upsert(
            tx.as_mut(),
            user.id(),
            "hashed:second",
            fixed_timestamp(1_700_000_800),
        )
        .await
        .unwrap();
    transactions.commit(tx).await.unwrap();

    let row = sqlx::query(
        "SELECT password_hash, created_at, updated_at FROM identity.user_credentials WHERE user_id = $1",
    )
    .bind(Uuid::parse_str(user.id().as_str()).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(row.get::<String, _>("password_hash"), "hashed:second");
    assert_eq!(
        row.get::<OffsetDateTime, _>("created_at"),
        fixed_timestamp(1_700_000_700)
    );
    assert_eq!(
        row.get::<OffsetDateTime, _>("updated_at"),
        fixed_timestamp(1_700_000_800)
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_credential_repository_rejects_invalid_uuid_string(pool: PgPool) {
    let repository = SqlxCredentialRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let mut tx = transactions.begin().await.unwrap();
    let error = repository
        .find_by_user_id(tx.as_mut(), &UserId::new("not-a-uuid"))
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::Validation { ref message } if message == "user id must be a valid UUID")
    );
}
