use ordering_food_authz_application::{AuthorizationRepository, AuthorizationService};
use ordering_food_authz_domain::{GlobalRole, StoreRole};
use ordering_food_authz_infrastructure_sqlx::SqlxAuthorizationRepository;
use sqlx::{PgPool, Row};
use time::macros::datetime;
use uuid::Uuid;

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn authorization_repository_returns_empty_roles_when_none_exist(pool: PgPool) {
    let repository = SqlxAuthorizationRepository::new(pool);
    let user_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();

    assert!(
        repository
            .get_global_roles(&user_id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        repository
            .get_store_roles(&user_id, &store_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn authorization_repository_rejects_invalid_uuid_inputs(pool: PgPool) {
    let repository = SqlxAuthorizationRepository::new(pool);

    let global_error = repository.get_global_roles("not-a-uuid").await.unwrap_err();
    let store_error = repository
        .get_store_roles("not-a-uuid", "also-not-a-uuid")
        .await
        .unwrap_err();

    assert!(
        global_error
            .to_string()
            .contains("user id must be a valid UUID")
    );
    assert!(
        store_error
            .to_string()
            .contains("user id must be a valid UUID")
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn authorization_repository_reads_enum_roles(pool: PgPool) {
    let repository = SqlxAuthorizationRepository::new(pool.clone());
    let user_id = Uuid::now_v7();
    let store_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO authz.user_global_roles (user_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(user_id)
    .bind(datetime!(2026-03-15 02:00 UTC))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO authz.store_memberships (user_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_staff', $3)
        "#,
    )
    .bind(user_id)
    .bind(store_id)
    .bind(datetime!(2026-03-15 02:01 UTC))
    .execute(&pool)
    .await
    .unwrap();

    let global_roles = repository
        .get_global_roles(&user_id.to_string())
        .await
        .unwrap();
    let store_roles = repository
        .get_store_roles(&user_id.to_string(), &store_id.to_string())
        .await
        .unwrap();

    assert_eq!(global_roles, vec![GlobalRole::PlatformAdmin]);
    assert_eq!(store_roles, vec![StoreRole::StoreStaff]);
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn authorization_service_allows_store_membership_and_platform_admin(pool: PgPool) {
    let repository = SqlxAuthorizationRepository::new(pool.clone());
    let service = AuthorizationService::new(std::sync::Arc::new(repository));
    let platform_admin = Uuid::now_v7();
    let staff_user = Uuid::now_v7();
    let store_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO authz.user_global_roles (user_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(platform_admin)
    .bind(datetime!(2026-03-15 02:02 UTC))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO authz.store_memberships (user_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_owner', $3)
        "#,
    )
    .bind(staff_user)
    .bind(store_id)
    .bind(datetime!(2026-03-15 02:03 UTC))
    .execute(&pool)
    .await
    .unwrap();

    assert!(
        service
            .can_manage_order(&platform_admin.to_string(), &Uuid::now_v7().to_string())
            .await
            .unwrap()
    );
    assert!(
        service
            .can_manage_order(&staff_user.to_string(), &store_id.to_string())
            .await
            .unwrap()
    );
    assert!(
        !service
            .can_manage_order(&staff_user.to_string(), &Uuid::now_v7().to_string())
            .await
            .unwrap()
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn authz_migration_creates_expected_schema_tables_enums_and_indexes(pool: PgPool) {
    let schema_exists: bool = sqlx::query_scalar(
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
    assert!(schema_exists);

    let type_names = sqlx::query(
        r#"
        SELECT t.typname
        FROM pg_type t
        INNER JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE n.nspname = 'authz'
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
        WHERE table_schema = 'authz'
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("table_name"))
    .collect::<Vec<_>>();
    assert_eq!(tables, vec!["store_memberships", "user_global_roles"]);

    let indexes = sqlx::query(
        r#"
        SELECT indexname
        FROM pg_indexes
        WHERE schemaname = 'authz'
          AND indexname IN (
            'idx_authz_user_global_roles_user_id',
            'idx_authz_store_memberships_user_store',
            'idx_authz_store_memberships_store_role'
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
            "idx_authz_store_memberships_store_role",
            "idx_authz_store_memberships_user_store",
            "idx_authz_user_global_roles_user_id",
        ]
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn authz_tables_reject_duplicate_role_assignments(pool: PgPool) {
    let user_id = Uuid::now_v7();
    let store_id = Uuid::now_v7();
    let granted_at = datetime!(2026-03-15 03:00 UTC);

    sqlx::query(
        r#"
        INSERT INTO authz.user_global_roles (user_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(user_id)
    .bind(granted_at)
    .execute(&pool)
    .await
    .unwrap();

    let duplicate_global = sqlx::query(
        r#"
        INSERT INTO authz.user_global_roles (user_id, role, granted_at)
        VALUES ($1, 'platform_admin', $2)
        "#,
    )
    .bind(user_id)
    .bind(granted_at)
    .execute(&pool)
    .await;
    assert!(duplicate_global.unwrap_err().as_database_error().is_some());

    sqlx::query(
        r#"
        INSERT INTO authz.store_memberships (user_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_staff', $3)
        "#,
    )
    .bind(user_id)
    .bind(store_id)
    .bind(granted_at)
    .execute(&pool)
    .await
    .unwrap();

    let duplicate_membership = sqlx::query(
        r#"
        INSERT INTO authz.store_memberships (user_id, store_id, role, granted_at)
        VALUES ($1, $2, 'store_staff', $3)
        "#,
    )
    .bind(user_id)
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
