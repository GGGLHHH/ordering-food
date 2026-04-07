use ordering_food_access_application::AccessGrantRepository;
use ordering_food_access_domain::AccessRole;
use ordering_food_access_infrastructure_sqlx::SqlxAccessGrantRepository;
use sqlx::{PgPool, Row};
use time::macros::datetime;
use uuid::Uuid;

const AUTHZ_UP_SQL: &str =
    include_str!("../../database-infrastructure-sqlx/migrations/202603150002_authz.up.sql",);
const ACCESS_UP_SQL: &str =
    include_str!("../../database-infrastructure-sqlx/migrations/202604050201_access.up.sql",);
const ACCESS_DOWN_SQL: &str =
    include_str!("../../database-infrastructure-sqlx/migrations/202604050201_access.down.sql",);

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn access_grant_repository_returns_empty_roles_when_none_exist(pool: PgPool) {
    let repository = SqlxAccessGrantRepository::new(pool);
    let subject_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();

    assert!(
        repository
            .get_platform_roles(&subject_id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        repository
            .get_store_roles(&subject_id, &store_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn access_grant_repository_rejects_invalid_uuid_inputs(pool: PgPool) {
    let repository = SqlxAccessGrantRepository::new(pool);
    let subject_id = Uuid::now_v7().to_string();

    let platform_error = repository
        .get_platform_roles("not-a-uuid")
        .await
        .unwrap_err();
    let store_error = repository
        .get_store_roles(&subject_id, "not-a-uuid")
        .await
        .unwrap_err();

    assert!(
        platform_error
            .to_string()
            .contains("subject id must be a valid UUID")
    );
    assert!(
        store_error
            .to_string()
            .contains("store id must be a valid UUID")
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn access_grant_repository_reads_enum_roles(pool: PgPool) {
    let repository = SqlxAccessGrantRepository::new(pool.clone());
    let subject_id = Uuid::now_v7();
    let store_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO access.subject_global_roles (subject_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(subject_id)
    .bind(datetime!(2026-04-05 02:10 UTC))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO access.store_memberships (subject_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_staff', $3)
        "#,
    )
    .bind(subject_id)
    .bind(store_id)
    .bind(datetime!(2026-04-05 02:11 UTC))
    .execute(&pool)
    .await
    .unwrap();

    let platform_roles = repository
        .get_platform_roles(&subject_id.to_string())
        .await
        .unwrap();
    let store_roles = repository
        .get_store_roles(&subject_id.to_string(), &store_id.to_string())
        .await
        .unwrap();

    assert_eq!(platform_roles, vec![AccessRole::PlatformAdmin]);
    assert_eq!(store_roles, vec![AccessRole::StoreStaff]);
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn access_migration_creates_expected_schema_tables_enums_and_indexes(pool: PgPool) {
    let schema_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.schemata
            WHERE schema_name = 'access'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(schema_exists);

    let type_names = sqlx::query(
        r#"
        SELECT t.typname
        FROM pg_type t
        INNER JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE n.nspname = 'access'
        ORDER BY t.typname
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("typname"))
    .collect::<Vec<_>>();
    assert!(type_names.contains(&"global_role".to_string()));
    assert!(type_names.contains(&"store_role".to_string()));

    let tables = sqlx::query(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'access'
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("table_name"))
    .collect::<Vec<_>>();
    assert_eq!(tables, vec!["store_memberships", "subject_global_roles"]);

    let indexes = sqlx::query(
        r#"
        SELECT indexname
        FROM pg_indexes
        WHERE schemaname = 'access'
          AND indexname IN (
            'idx_access_subject_global_roles_subject_id',
            'idx_access_store_memberships_subject_store',
            'idx_access_store_memberships_store_role'
          )
        ORDER BY indexname
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("indexname"))
    .collect::<Vec<_>>();
    assert_eq!(
        indexes,
        vec![
            "idx_access_store_memberships_store_role",
            "idx_access_store_memberships_subject_store",
            "idx_access_subject_global_roles_subject_id",
        ]
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn access_tables_reject_duplicate_role_assignments(pool: PgPool) {
    let subject_id = Uuid::now_v7();
    let store_id = Uuid::now_v7();
    let granted_at = datetime!(2026-04-05 03:00 UTC);

    sqlx::query(
        r#"
        INSERT INTO access.subject_global_roles (subject_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(subject_id)
    .bind(granted_at)
    .execute(&pool)
    .await
    .unwrap();

    let duplicate_global = sqlx::query(
        r#"
        INSERT INTO access.subject_global_roles (subject_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(subject_id)
    .bind(granted_at)
    .execute(&pool)
    .await;
    assert!(duplicate_global.unwrap_err().as_database_error().is_some());

    sqlx::query(
        r#"
        INSERT INTO access.store_memberships (subject_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_staff', $3)
        "#,
    )
    .bind(subject_id)
    .bind(store_id)
    .bind(granted_at)
    .execute(&pool)
    .await
    .unwrap();

    let duplicate_membership = sqlx::query(
        r#"
        INSERT INTO access.store_memberships (subject_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_staff', $3)
        "#,
    )
    .bind(subject_id)
    .bind(store_id)
    .bind(granted_at)
    .execute(&pool)
    .await;
    assert!(
        duplicate_membership
            .unwrap_err()
            .as_database_error()
            .is_some()
    );
}

#[sqlx::test]
async fn access_up_migration_copies_existing_authz_data(pool: PgPool) {
    let subject_id = Uuid::now_v7();
    let store_id = Uuid::now_v7();

    sqlx::raw_sql(AUTHZ_UP_SQL).execute(&pool).await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO authz.user_global_roles (user_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(subject_id)
    .bind(datetime!(2026-04-05 04:00 UTC))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO authz.store_memberships (user_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_owner', $3)
        "#,
    )
    .bind(subject_id)
    .bind(store_id)
    .bind(datetime!(2026-04-05 04:01 UTC))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::raw_sql(ACCESS_UP_SQL).execute(&pool).await.unwrap();

    let repository = SqlxAccessGrantRepository::new(pool);
    let platform_roles = repository
        .get_platform_roles(&subject_id.to_string())
        .await
        .unwrap();
    let store_roles = repository
        .get_store_roles(&subject_id.to_string(), &store_id.to_string())
        .await
        .unwrap();

    assert_eq!(platform_roles, vec![AccessRole::PlatformAdmin]);
    assert_eq!(store_roles, vec![AccessRole::StoreOwner]);
}

#[sqlx::test]
async fn access_down_migration_only_removes_access_schema(pool: PgPool) {
    sqlx::raw_sql(AUTHZ_UP_SQL).execute(&pool).await.unwrap();
    sqlx::raw_sql(ACCESS_UP_SQL).execute(&pool).await.unwrap();
    sqlx::raw_sql(ACCESS_DOWN_SQL).execute(&pool).await.unwrap();

    let authz_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.schemata
            WHERE schema_name = 'authz'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let authz_table_exists: bool = sqlx::query_scalar(
        r#"
        SELECT to_regclass('authz.user_global_roles') IS NOT NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let access_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.schemata
            WHERE schema_name = 'access'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(authz_exists);
    assert!(authz_table_exists);
    assert!(!access_exists);
}
