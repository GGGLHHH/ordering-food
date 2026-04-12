use ordering_food_app_support::{config::Settings, runtime::SystemClock};
use ordering_food_bootstrap::run_default_data_bootstrap;
use ordering_food_database_infrastructure_sqlx::MIGRATOR;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use time::macros::datetime;
use uuid::Uuid;

const DEFAULT_BRAND_ID: &str = "00000000-0000-4000-8000-000000000001";
const DEFAULT_STORE_SLUG: &str = "ordering-food-demo";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootstrapCounts {
    organization_brands: i64,
    organization_stores: i64,
    catalog_brand_catalogs: i64,
    catalog_store_catalogs: i64,
    catalog_categories: i64,
    catalog_items: i64,
    catalog_store_item_listings: i64,
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn bootstrap_seeds_default_store_even_when_another_active_store_exists(pool: PgPool) {
    insert_active_store(
        &pool,
        Uuid::now_v7(),
        Uuid::parse_str(DEFAULT_BRAND_ID).unwrap(),
        "other-active-store",
        datetime!(2026-04-05 03:00 UTC),
    )
    .await;

    let settings = Settings::from_overrides(std::iter::empty::<(String, String)>()).unwrap();

    run_default_data_bootstrap(&settings, pool.clone(), Arc::new(SystemClock))
        .await
        .unwrap();

    let default_store_id = load_store_id_by_slug(&pool, DEFAULT_STORE_SLUG)
        .await
        .expect("default store should exist after bootstrap");
    let default_store_catalogs = count_store_catalogs_for_slug(&pool, DEFAULT_STORE_SLUG).await;
    let default_store_listings =
        count_store_item_listings_for_slug(&pool, DEFAULT_STORE_SLUG).await;
    let other_store_catalogs = count_store_catalogs_for_slug(&pool, "other-active-store").await;

    assert!(!default_store_id.is_nil());
    assert_eq!(default_store_catalogs, 1);
    assert_eq!(default_store_listings, 8);
    assert_eq!(other_store_catalogs, 0);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn bootstrap_repeated_runs_do_not_duplicate_default_data(pool: PgPool) {
    let settings = Settings::from_overrides(std::iter::empty::<(String, String)>()).unwrap();

    run_default_data_bootstrap(&settings, pool.clone(), Arc::new(SystemClock))
        .await
        .unwrap();
    let first_counts = load_counts(&pool).await;

    run_default_data_bootstrap(&settings, pool.clone(), Arc::new(SystemClock))
        .await
        .unwrap();
    let second_counts = load_counts(&pool).await;

    assert_eq!(
        first_counts,
        BootstrapCounts {
            organization_brands: 1,
            organization_stores: 1,
            catalog_brand_catalogs: 1,
            catalog_store_catalogs: 1,
            catalog_categories: 4,
            catalog_items: 8,
            catalog_store_item_listings: 8,
        }
    );
    assert_eq!(second_counts, first_counts);
}

async fn insert_active_store(
    pool: &PgPool,
    store_id: Uuid,
    brand_id: Uuid,
    slug: &str,
    created_at: time::OffsetDateTime,
) {
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
    .bind(brand_id)
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO organization.stores (
            id,
            brand_id,
            slug,
            name,
            currency_code,
            timezone,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, $3, 'Other Active Store', 'CNY', 'Asia/Shanghai', 'active', $4, $4, NULL)
        "#,
    )
    .bind(store_id)
    .bind(brand_id)
    .bind(slug)
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();
}

async fn load_store_id_by_slug(pool: &PgPool, slug: &str) -> Option<Uuid> {
    sqlx::query_scalar(
        r#"
        SELECT id
        FROM organization.stores
        WHERE slug = $1 AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .unwrap()
}

async fn count_store_catalogs_for_slug(pool: &PgPool, slug: &str) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM catalog.store_catalogs AS store_catalogs
        INNER JOIN organization.stores AS stores
            ON stores.id = store_catalogs.store_id
        WHERE stores.slug = $1
        "#,
    )
    .bind(slug)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn count_store_item_listings_for_slug(pool: &PgPool, slug: &str) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM catalog.store_item_listings AS listings
        INNER JOIN catalog.store_catalogs AS store_catalogs
            ON store_catalogs.id = listings.store_catalog_id
        INNER JOIN organization.stores AS stores
            ON stores.id = store_catalogs.store_id
        WHERE stores.slug = $1
        "#,
    )
    .bind(slug)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn load_counts(pool: &PgPool) -> BootstrapCounts {
    let organization_brands = count_rows(pool, "organization.brands").await;
    let organization_stores = count_rows(pool, "organization.stores").await;
    let catalog_brand_catalogs = count_rows(pool, "catalog.brand_catalogs").await;
    let catalog_store_catalogs = count_rows(pool, "catalog.store_catalogs").await;
    let catalog_categories = count_rows(pool, "catalog.categories").await;
    let catalog_items = count_rows(pool, "catalog.items").await;
    let catalog_store_item_listings = count_rows(pool, "catalog.store_item_listings").await;

    BootstrapCounts {
        organization_brands,
        organization_stores,
        catalog_brand_catalogs,
        catalog_store_catalogs,
        catalog_categories,
        catalog_items,
        catalog_store_item_listings,
    }
}

async fn count_rows(pool: &PgPool, table_name: &str) -> i64 {
    let query = format!("SELECT COUNT(*) AS count FROM {table_name}");
    sqlx::query(&query)
        .fetch_one(pool)
        .await
        .unwrap()
        .get::<i64, _>("count")
}
