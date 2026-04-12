use ordering_food_database_infrastructure_sqlx::MIGRATOR;
use ordering_food_organization_application::{
    BrandReadRepository, OrganizationUnitOfWorkFactory, StoreReadRepository,
};
use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus, Store, StoreId};
use ordering_food_organization_infrastructure_sqlx::{
    SqlxBrandReadRepository, SqlxOrganizationUnitOfWorkFactory, SqlxStoreReadRepository,
};
use ordering_food_shared_kernel::Timestamp;
use sqlx::{PgPool, types::time::OffsetDateTime};
use uuid::Uuid;

fn unique_uuid() -> Uuid {
    Uuid::now_v7()
}

fn fixed_timestamp(seconds: i64) -> Timestamp {
    OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}

fn default_brand_id() -> BrandId {
    BrandId::new("00000000-0000-4000-8000-000000000001")
}

fn default_brand_uuid() -> Uuid {
    Uuid::parse_str(default_brand_id().as_str()).unwrap()
}

fn make_store(store_id: Uuid, created_at: Timestamp) -> Store {
    Store::create(
        StoreId::new(store_id.to_string()),
        default_brand_id(),
        format!("demo-store-{store_id}"),
        "Demo Store",
        "CNY",
        "Asia/Shanghai",
        OrganizationStatus::Active,
        created_at,
    )
    .unwrap()
}

fn make_brand(brand_id: Uuid, created_at: Timestamp) -> Brand {
    Brand::create(
        BrandId::new(brand_id.to_string()),
        format!("demo-brand-{brand_id}"),
        "Demo Brand",
        OrganizationStatus::Active,
        created_at,
    )
    .unwrap()
}

async fn insert_brand(pool: &PgPool, brand: &Brand) {
    let unit_of_work_factory = SqlxOrganizationUnitOfWorkFactory::new(pool.clone());
    let mut unit_of_work = unit_of_work_factory.begin().await.unwrap();
    unit_of_work.insert_brand(brand).await.unwrap();
    unit_of_work.commit().await.unwrap();
}

async fn ensure_default_brand(pool: &PgPool) {
    sqlx::query(
        r#"
        INSERT INTO organization.brands (
            id,
            slug,
            name,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, 'ordering-food', 'Ordering Food', 'active', $2, $2, NULL)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(default_brand_uuid())
    .bind(fixed_timestamp(1_700_000_000))
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_store(pool: &PgPool, store: &Store) {
    ensure_default_brand(pool).await;

    let unit_of_work_factory = SqlxOrganizationUnitOfWorkFactory::new(pool.clone());
    let mut unit_of_work = unit_of_work_factory.begin().await.unwrap();
    unit_of_work.insert_store(store).await.unwrap();
    unit_of_work.commit().await.unwrap();
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn organization_migration_creates_schema_tables(pool: PgPool) {
    let schemas = sqlx::query_scalar::<_, String>(
        r#"
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name = 'organization'
        ORDER BY schema_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(schemas, vec!["organization".to_string()]);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn organization_migration_does_not_seed_default_business_data(pool: PgPool) {
    let brand_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organization.brands")
        .fetch_one(&pool)
        .await
        .unwrap();
    let store_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organization.stores")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(brand_count, 0);
    assert_eq!(store_count, 0);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_unit_of_work_inserts_and_loads_brand(pool: PgPool) {
    let brand = Brand::create(
        BrandId::new(unique_uuid().to_string()),
        "new-brand",
        "New Brand",
        OrganizationStatus::Active,
        fixed_timestamp(1_700_000_000),
    )
    .unwrap();
    let unit_of_work_factory = SqlxOrganizationUnitOfWorkFactory::new(pool);
    let mut unit_of_work = unit_of_work_factory.begin().await.unwrap();
    unit_of_work.insert_brand(&brand).await.unwrap();
    let loaded = unit_of_work.find_brand_by_id(brand.id()).await.unwrap();

    assert_eq!(loaded.unwrap().id(), brand.id());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_unit_of_work_inserts_and_loads_store_by_brand_slug(pool: PgPool) {
    let store = make_store(unique_uuid(), fixed_timestamp(1_700_000_100));
    ensure_default_brand(&pool).await;

    let unit_of_work_factory = SqlxOrganizationUnitOfWorkFactory::new(pool);
    let mut unit_of_work = unit_of_work_factory.begin().await.unwrap();
    unit_of_work.insert_store(&store).await.unwrap();
    let loaded = unit_of_work
        .find_store_by_brand_slug(store.brand_id(), store.slug())
        .await
        .unwrap();

    assert_eq!(loaded.unwrap().id(), store.id());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_store_read_repository_returns_active_store_summary_from_menu_view(pool: PgPool) {
    insert_store(
        &pool,
        &make_store(unique_uuid(), fixed_timestamp(1_700_000_100)),
    )
    .await;

    let repository = SqlxStoreReadRepository::new(pool);
    let active_store = repository.get_active().await.unwrap().unwrap();

    assert_eq!(
        active_store.brand_id,
        default_brand_id().as_str().to_string()
    );
    assert_eq!(active_store.currency_code, "CNY");
    assert_eq!(active_store.timezone, "Asia/Shanghai");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_brand_read_repository_returns_brand_ref_by_id(pool: PgPool) {
    let brand_id = unique_uuid();
    insert_brand(&pool, &make_brand(brand_id, fixed_timestamp(1_700_000_050))).await;

    let repository = SqlxBrandReadRepository::new(pool);
    let brand = repository
        .get_by_id(&brand_id.to_string())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(brand.brand_id, brand_id.to_string());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_brand_read_repository_rejects_invalid_uuid_inputs(pool: PgPool) {
    let repository = SqlxBrandReadRepository::new(pool);

    let error = repository.get_by_id("not-a-uuid").await.unwrap_err();

    assert!(error.to_string().contains("brand id must be a valid UUID"));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_brand_read_repository_skips_deleted_brand(pool: PgPool) {
    let brand_id = unique_uuid();
    insert_brand(&pool, &make_brand(brand_id, fixed_timestamp(1_700_000_060))).await;

    sqlx::query(
        r#"
        UPDATE organization.brands
        SET deleted_at = $2
        WHERE id = $1
        "#,
    )
    .bind(brand_id)
    .bind(fixed_timestamp(1_700_000_061))
    .execute(&pool)
    .await
    .unwrap();

    let repository = SqlxBrandReadRepository::new(pool);
    let brand = repository.get_by_id(&brand_id.to_string()).await.unwrap();

    assert!(brand.is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn get_by_id_returns_store_when_store_exists(pool: PgPool) {
    let store_id = unique_uuid();
    insert_store(&pool, &make_store(store_id, fixed_timestamp(1_700_000_200))).await;

    let repository = SqlxStoreReadRepository::new(pool);
    let store = repository
        .get_by_id(&store_id.to_string())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(store.store_id, store_id.to_string());
    assert_eq!(store.brand_id, default_brand_id().as_str().to_string());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn get_by_id_rejects_invalid_uuid_inputs(pool: PgPool) {
    let repository = SqlxStoreReadRepository::new(pool);

    let error = repository.get_by_id("not-a-uuid").await.unwrap_err();

    assert!(error.to_string().contains("store id must be a valid UUID"));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn get_by_id_skips_deleted_store(pool: PgPool) {
    let store_id = unique_uuid();
    insert_store(&pool, &make_store(store_id, fixed_timestamp(1_700_000_300))).await;

    sqlx::query(
        r#"
        UPDATE organization.stores
        SET deleted_at = $2
        WHERE id = $1
        "#,
    )
    .bind(store_id)
    .bind(fixed_timestamp(1_700_000_301))
    .execute(&pool)
    .await
    .unwrap();

    let repository = SqlxStoreReadRepository::new(pool);
    let store = repository.get_by_id(&store_id.to_string()).await.unwrap();

    assert!(store.is_none());
}
