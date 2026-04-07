use ordering_food_catalog_application::{
    BrandCatalogRepository, CategoryRepository, ItemRepository, StoreCatalogRepository,
    StoreItemListingRepository, TransactionManager,
};
use ordering_food_catalog_domain::{
    BrandCatalog, BrandCatalogId, BrandId, Category, CategoryId, DisplayRule, Item, ItemId, Price,
    SellableStatus, StoreCatalog, StoreCatalogId, StoreId, StoreItemListing,
};
use ordering_food_catalog_infrastructure_sqlx::{
    SqlxBrandCatalogReadRepository, SqlxBrandCatalogRepository, SqlxCategoryReadRepository,
    SqlxCategoryRepository, SqlxItemReadRepository, SqlxItemRepository,
    SqlxStoreCatalogReadRepository, SqlxStoreCatalogRepository, SqlxStoreItemListingRepository,
    SqlxTransactionManager,
};
use ordering_food_database_infrastructure_sqlx::MIGRATOR;
use sqlx::{PgPool, Row};
use time::macros::datetime;
use uuid::Uuid;

const BASELINE_UP_SQL: &str =
    include_str!("../../database-infrastructure-sqlx/migrations/202603140001_baseline.up.sql");
const ORGANIZATION_UP_SQL: &str = include_str!(
    "../../database-infrastructure-sqlx/migrations/202604050101_organization_foundation.up.sql",
);
const CATALOG_UP_SQL: &str = include_str!(
    "../../database-infrastructure-sqlx/migrations/202604050301_catalog_context.up.sql",
);

fn unique_uuid() -> Uuid {
    Uuid::now_v7()
}

async fn insert_brand_scope(pool: &PgPool, brand_id: Uuid, created_at: time::OffsetDateTime) {
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
        VALUES ($1, $2, $3, 'active', $4, $4, NULL)
        "#,
    )
    .bind(brand_id)
    .bind(format!("brand-{brand_id}"))
    .bind("Demo Brand")
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_store_scope(
    pool: &PgPool,
    brand_id: Uuid,
    store_id: Uuid,
    slug: &str,
    created_at: time::OffsetDateTime,
) {
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
        VALUES ($1, $2, $3, 'Demo Store', 'CNY', 'Asia/Shanghai', 'active', $4, $4, NULL)
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

async fn insert_brand_catalog(
    pool: &PgPool,
    brand_catalog: &BrandCatalog,
) -> SqlxTransactionManager {
    let transactions = SqlxTransactionManager::new(pool.clone());
    let repository = SqlxBrandCatalogRepository;
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), brand_catalog).await.unwrap();
    transactions.commit(tx).await.unwrap();
    transactions
}

async fn insert_store_catalog(pool: &PgPool, store_catalog: &StoreCatalog) {
    let repository = SqlxStoreCatalogRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), store_catalog).await.unwrap();
    transactions.commit(tx).await.unwrap();
}

async fn insert_category(pool: &PgPool, category: &Category) {
    let repository = SqlxCategoryRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), category).await.unwrap();
    transactions.commit(tx).await.unwrap();
}

async fn insert_item(pool: &PgPool, item: &Item) {
    let repository = SqlxItemRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), item).await.unwrap();
    transactions.commit(tx).await.unwrap();
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn catalog_schema_contains_expected_tables_and_indexes(pool: PgPool) {
    let tables = sqlx::query(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'catalog'
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("table_name"))
    .collect::<Vec<_>>();

    assert_eq!(
        tables,
        vec![
            "brand_catalogs",
            "categories",
            "items",
            "store_catalogs",
            "store_item_listings",
        ]
    );

    let indexes = sqlx::query(
        r#"
        SELECT indexname
        FROM pg_indexes
        WHERE schemaname = 'catalog'
        ORDER BY indexname
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("indexname"))
    .collect::<Vec<_>>();

    for expected in [
        "idx_catalog_brand_catalogs_brand_id",
        "idx_catalog_categories_brand_catalog_sort",
        "idx_catalog_items_brand_catalog_sort",
        "idx_catalog_items_category_sort",
        "idx_catalog_store_catalogs_store_id",
        "idx_catalog_store_item_listings_store_catalog",
    ] {
        assert!(
            indexes.contains(&expected.to_string()),
            "missing catalog index {expected}"
        );
    }
}

#[sqlx::test]
async fn catalog_migration_backfills_existing_menu_data(pool: PgPool) {
    let store_id = unique_uuid();
    let category_id = unique_uuid();
    let item_id = unique_uuid();
    let now = datetime!(2026-04-05 04:00 UTC);
    let legacy_schema = "menu";

    sqlx::raw_sql(BASELINE_UP_SQL).execute(&pool).await.unwrap();

    sqlx::query(&format!(
        r#"
        INSERT INTO {legacy_schema}.stores (
            id,
            slug,
            name,
            currency_code,
            timezone,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, 'demo-store', 'Demo Store', 'CNY', 'Asia/Shanghai', 'active', $2, $2, NULL)
        "#
    ))
    .bind(store_id)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(&format!(
        r#"
        INSERT INTO {legacy_schema}.categories (
            id,
            store_id,
            slug,
            name,
            description,
            sort_order,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, 'featured', 'Featured', NULL, 10, 'active', $3, $3, NULL)
        "#
    ))
    .bind(category_id)
    .bind(store_id)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(&format!(
        r#"
        INSERT INTO {legacy_schema}.items (
            id,
            store_id,
            category_id,
            slug,
            name,
            description,
            image_url,
            price_amount,
            sort_order,
            status,
            created_at,
            updated_at,
            deleted_at
        )
        VALUES ($1, $2, $3, 'crispy-bowl', 'Crispy Bowl', NULL, NULL, 3200, 20, 'active', $4, $4, NULL)
        "#
    ))
    .bind(item_id)
    .bind(store_id)
    .bind(category_id)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::raw_sql(ORGANIZATION_UP_SQL)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(CATALOG_UP_SQL).execute(&pool).await.unwrap();

    let brand_catalog_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM catalog.brand_catalogs")
            .fetch_one(&pool)
            .await
            .unwrap();
    let store_catalog_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM catalog.store_catalogs")
            .fetch_one(&pool)
            .await
            .unwrap();
    let category_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog.categories")
        .fetch_one(&pool)
        .await
        .unwrap();
    let item_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog.items")
        .fetch_one(&pool)
        .await
        .unwrap();
    let listing_row = sqlx::query(
        r#"
        SELECT price_amount, status, display_rule
        FROM catalog.store_item_listings
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(brand_catalog_count, 1);
    assert_eq!(store_catalog_count, 1);
    assert_eq!(category_count, 1);
    assert_eq!(item_count, 1);
    assert_eq!(listing_row.get::<i64, _>("price_amount"), 3200);
    assert_eq!(listing_row.get::<String, _>("status"), "sellable");
    assert_eq!(listing_row.get::<String, _>("display_rule"), "listed");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_brand_catalog_repository_rejects_duplicate_brand_scope(pool: PgPool) {
    let brand_id = unique_uuid();
    insert_brand_scope(&pool, brand_id, datetime!(2026-04-05 04:05 UTC)).await;

    let repository = SqlxBrandCatalogRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let first = BrandCatalog::create(
        BrandCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        "demo-catalog",
        "Demo Catalog",
        datetime!(2026-04-05 04:06 UTC),
    )
    .unwrap();
    let duplicate = BrandCatalog::create(
        BrandCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        "duplicate-catalog",
        "Duplicate Catalog",
        datetime!(2026-04-05 04:07 UTC),
    )
    .unwrap();

    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), &first).await.unwrap();
    let error = repository
        .insert(tx.as_mut(), &duplicate)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ordering_food_catalog_application::ApplicationError::Conflict { ref message } if message == "brand catalog already exists for brand scope")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_store_catalog_repository_rejects_duplicate_store_scope(pool: PgPool) {
    let brand_id = unique_uuid();
    let store_id = unique_uuid();
    insert_brand_scope(&pool, brand_id, datetime!(2026-04-05 04:08 UTC)).await;
    insert_store_scope(
        &pool,
        brand_id,
        store_id,
        "demo-store",
        datetime!(2026-04-05 04:09 UTC),
    )
    .await;

    let repository = SqlxStoreCatalogRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let first = StoreCatalog::attach(
        StoreCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        StoreId::new(store_id.to_string()),
        SellableStatus::Sellable,
        DisplayRule::listed(),
        datetime!(2026-04-05 04:10 UTC),
    )
    .unwrap();
    let duplicate = StoreCatalog::attach(
        StoreCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        StoreId::new(store_id.to_string()),
        SellableStatus::Unsellable,
        DisplayRule::hidden(),
        datetime!(2026-04-05 04:11 UTC),
    )
    .unwrap();

    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), &first).await.unwrap();
    let error = repository
        .insert(tx.as_mut(), &duplicate)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ordering_food_catalog_application::ApplicationError::Conflict { ref message } if message == "store catalog already exists for store scope")
    );
}

#[sqlx::test]
async fn catalog_migration_merges_duplicate_store_level_slugs_into_brand_level_catalog(
    pool: PgPool,
) {
    let store_a = unique_uuid();
    let store_b = unique_uuid();
    let category_a = unique_uuid();
    let category_b = unique_uuid();
    let item_a = unique_uuid();
    let item_b = unique_uuid();
    let now = datetime!(2026-04-05 04:12 UTC);
    let legacy_schema = "menu";

    sqlx::raw_sql(BASELINE_UP_SQL).execute(&pool).await.unwrap();

    for (store_id, slug) in [(store_a, "demo-store-a"), (store_b, "demo-store-b")] {
        sqlx::query(&format!(
            r#"
            INSERT INTO {legacy_schema}.stores (
                id,
                slug,
                name,
                currency_code,
                timezone,
                status,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, 'Demo Store', 'CNY', 'Asia/Shanghai', 'active', $3, $3, NULL)
            "#
        ))
        .bind(store_id)
        .bind(slug)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    }

    for (category_id, store_id) in [(category_a, store_a), (category_b, store_b)] {
        sqlx::query(&format!(
            r#"
            INSERT INTO {legacy_schema}.categories (
                id,
                store_id,
                slug,
                name,
                description,
                sort_order,
                status,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, 'featured', 'Featured', NULL, 10, 'active', $3, $3, NULL)
            "#
        ))
        .bind(category_id)
        .bind(store_id)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    }

    for (item_id, store_id, category_id, price_amount) in [
        (item_a, store_a, category_a, 3200_i64),
        (item_b, store_b, category_b, 3600_i64),
    ] {
        sqlx::query(&format!(
            r#"
            INSERT INTO {legacy_schema}.items (
                id,
                store_id,
                category_id,
                slug,
                name,
                description,
                image_url,
                price_amount,
                sort_order,
                status,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, 'crispy-bowl', 'Crispy Bowl', NULL, NULL, $4, 20, 'active', $5, $5, NULL)
            "#
        ))
        .bind(item_id)
        .bind(store_id)
        .bind(category_id)
        .bind(price_amount)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    }

    sqlx::raw_sql(ORGANIZATION_UP_SQL)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(CATALOG_UP_SQL).execute(&pool).await.unwrap();

    let category_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog.categories")
        .fetch_one(&pool)
        .await
        .unwrap();
    let item_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog.items")
        .fetch_one(&pool)
        .await
        .unwrap();
    let listing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog.store_item_listings")
        .fetch_one(&pool)
        .await
        .unwrap();
    let distinct_item_ids: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT item_id) FROM catalog.store_item_listings")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(category_count, 1);
    assert_eq!(item_count, 1);
    assert_eq!(listing_count, 2);
    assert_eq!(distinct_item_ids, 1);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_category_repository_rejects_duplicate_slug_within_brand_catalog(pool: PgPool) {
    let brand_id = unique_uuid();
    insert_brand_scope(&pool, brand_id, datetime!(2026-04-05 04:10 UTC)).await;

    let brand_catalog = BrandCatalog::create(
        BrandCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        "demo-catalog",
        "Demo Catalog",
        datetime!(2026-04-05 04:11 UTC),
    )
    .unwrap();
    insert_brand_catalog(&pool, &brand_catalog).await;

    let repository = SqlxCategoryRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let first = Category::create(
        CategoryId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        "featured",
        "Featured",
        None,
        10,
        datetime!(2026-04-05 04:12 UTC),
    )
    .unwrap();
    let duplicate = Category::create(
        CategoryId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        "featured",
        "Other Featured",
        None,
        20,
        datetime!(2026-04-05 04:13 UTC),
    )
    .unwrap();

    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), &first).await.unwrap();
    let error = repository
        .insert(tx.as_mut(), &duplicate)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ordering_food_catalog_application::ApplicationError::Conflict { ref message } if message == "category slug already exists in brand catalog")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_item_repository_rejects_duplicate_slug_within_brand_catalog(pool: PgPool) {
    let brand_id = unique_uuid();
    insert_brand_scope(&pool, brand_id, datetime!(2026-04-05 04:20 UTC)).await;

    let brand_catalog = BrandCatalog::create(
        BrandCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        "demo-catalog",
        "Demo Catalog",
        datetime!(2026-04-05 04:21 UTC),
    )
    .unwrap();
    insert_brand_catalog(&pool, &brand_catalog).await;

    let mains = Category::create(
        CategoryId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        "mains",
        "Mains",
        None,
        10,
        datetime!(2026-04-05 04:22 UTC),
    )
    .unwrap();
    let drinks = Category::create(
        CategoryId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        "drinks",
        "Drinks",
        None,
        20,
        datetime!(2026-04-05 04:23 UTC),
    )
    .unwrap();
    insert_category(&pool, &mains).await;
    insert_category(&pool, &drinks).await;

    let repository = SqlxItemRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let first = Item::create(
        ItemId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        mains.id().clone(),
        "crispy-bowl",
        "Crispy Bowl",
        None,
        None,
        10,
        datetime!(2026-04-05 04:24 UTC),
    )
    .unwrap();
    let duplicate = Item::create(
        ItemId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        drinks.id().clone(),
        "crispy-bowl",
        "Another Bowl",
        None,
        None,
        20,
        datetime!(2026-04-05 04:25 UTC),
    )
    .unwrap();

    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), &first).await.unwrap();
    let error = repository
        .insert(tx.as_mut(), &duplicate)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ordering_food_catalog_application::ApplicationError::Conflict { ref message } if message == "item slug already exists in brand catalog")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_store_item_listing_repository_upserts_price_status_and_display_rule(pool: PgPool) {
    let brand_id = unique_uuid();
    let store_id = unique_uuid();
    insert_brand_scope(&pool, brand_id, datetime!(2026-04-05 04:30 UTC)).await;
    insert_store_scope(
        &pool,
        brand_id,
        store_id,
        "demo-store",
        datetime!(2026-04-05 04:31 UTC),
    )
    .await;

    let brand_catalog = BrandCatalog::create(
        BrandCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        "demo-catalog",
        "Demo Catalog",
        datetime!(2026-04-05 04:32 UTC),
    )
    .unwrap();
    insert_brand_catalog(&pool, &brand_catalog).await;

    let store_catalog = StoreCatalog::attach(
        StoreCatalogId::new(unique_uuid().to_string()),
        BrandId::new(brand_id.to_string()),
        StoreId::new(store_id.to_string()),
        SellableStatus::Sellable,
        DisplayRule::listed(),
        datetime!(2026-04-05 04:33 UTC),
    )
    .unwrap();
    insert_store_catalog(&pool, &store_catalog).await;

    let category = Category::create(
        CategoryId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        "featured",
        "Featured",
        None,
        10,
        datetime!(2026-04-05 04:34 UTC),
    )
    .unwrap();
    insert_category(&pool, &category).await;

    let item = Item::create(
        ItemId::new(unique_uuid().to_string()),
        brand_catalog.id().clone(),
        category.id().clone(),
        "crispy-bowl",
        "Crispy Bowl",
        None,
        None,
        10,
        datetime!(2026-04-05 04:35 UTC),
    )
    .unwrap();
    insert_item(&pool, &item).await;

    let repository = SqlxStoreItemListingRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());

    let mut tx = transactions.begin().await.unwrap();
    repository
        .upsert(
            tx.as_mut(),
            &StoreItemListing::upsert(
                store_catalog.id().clone(),
                item.id().clone(),
                Price::new(3200).unwrap(),
                SellableStatus::Sellable,
                DisplayRule::listed(),
                datetime!(2026-04-05 04:36 UTC),
            ),
        )
        .await
        .unwrap();
    transactions.commit(tx).await.unwrap();

    let mut tx = transactions.begin().await.unwrap();
    repository
        .upsert(
            tx.as_mut(),
            &StoreItemListing::upsert(
                store_catalog.id().clone(),
                item.id().clone(),
                Price::new(3600).unwrap(),
                SellableStatus::Unsellable,
                DisplayRule::hidden(),
                datetime!(2026-04-05 04:37 UTC),
            ),
        )
        .await
        .unwrap();
    transactions.commit(tx).await.unwrap();

    let listing_row = sqlx::query(
        r#"
        SELECT price_amount, status, display_rule
        FROM catalog.store_item_listings
        WHERE store_catalog_id = $1 AND item_id = $2
        "#,
    )
    .bind(Uuid::parse_str(store_catalog.id().as_str()).unwrap())
    .bind(Uuid::parse_str(item.id().as_str()).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(listing_row.get::<i64, _>("price_amount"), 3600);
    assert_eq!(listing_row.get::<String, _>("status"), "unsellable");
    assert_eq!(listing_row.get::<String, _>("display_rule"), "hidden");

    let brand_catalog_read = SqlxBrandCatalogReadRepository::new(pool.clone());
    let store_catalog_read = SqlxStoreCatalogReadRepository::new(pool.clone());
    let category_read = SqlxCategoryReadRepository::new(pool.clone());
    let item_read = SqlxItemReadRepository::new(pool.clone());

    let brand_catalog_model = brand_catalog_read
        .find_by_brand_id(brand_catalog.brand_id().as_str())
        .await
        .unwrap()
        .unwrap();
    let store_catalog_model = store_catalog_read
        .find_by_store_id(store_catalog.store_id().as_str())
        .await
        .unwrap()
        .unwrap();
    let categories = category_read
        .list_by_brand_catalog_id(brand_catalog.id().as_str())
        .await
        .unwrap();
    let items = item_read
        .list_by_brand_catalog_id(brand_catalog.id().as_str(), Some(category.id().as_str()))
        .await
        .unwrap();

    assert_eq!(brand_catalog_model.brand_id, brand_id.to_string());
    assert_eq!(store_catalog_model.store_id, store_id.to_string());
    assert_eq!(categories.len(), 1);
    assert_eq!(items.len(), 1);
}
