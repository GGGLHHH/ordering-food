use ordering_food_database_infrastructure_sqlx::MIGRATOR;
use ordering_food_menu_application::{
    ApplicationError, CategoryReadRepository, CategoryRepository, ItemListFilter,
    ItemReadRepository, ItemRepository, StoreReadRepository, StoreRepository, TransactionManager,
};
use ordering_food_menu_domain::{Category, CategoryId, Item, ItemId, MenuStatus, Store, StoreId};
use ordering_food_menu_infrastructure_sqlx::{
    SqlxCategoryReadRepository, SqlxCategoryRepository, SqlxItemReadRepository, SqlxItemRepository,
    SqlxStoreReadRepository, SqlxStoreRepository, SqlxTransactionManager,
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

async fn insert_store(pool: &PgPool, store: &Store) {
    let repository = SqlxStoreRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), store).await.unwrap();
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

fn make_store(store_id: Uuid, created_at: Timestamp) -> Store {
    Store::create(
        StoreId::new(store_id.to_string()),
        format!("demo-store-{store_id}"),
        "Demo Store",
        "CNY",
        "Asia/Shanghai",
        MenuStatus::Active,
        created_at,
    )
    .unwrap()
}

fn make_category(
    category_id: Uuid,
    store_id: Uuid,
    slug: &str,
    sort_order: i32,
    status: MenuStatus,
    created_at: Timestamp,
) -> Category {
    Category::create(
        CategoryId::new(category_id.to_string()),
        StoreId::new(store_id.to_string()),
        slug,
        slug,
        None,
        sort_order,
        status,
        created_at,
    )
    .unwrap()
}

fn make_item(
    item_id: Uuid,
    store_id: Uuid,
    category_id: Uuid,
    slug: &str,
    price_amount: i64,
    sort_order: i32,
    status: MenuStatus,
    created_at: Timestamp,
) -> Item {
    Item::create(
        ItemId::new(item_id.to_string()),
        StoreId::new(store_id.to_string()),
        CategoryId::new(category_id.to_string()),
        slug,
        slug,
        None,
        None,
        price_amount,
        sort_order,
        status,
        created_at,
    )
    .unwrap()
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_category_repository_rejects_duplicate_slug_within_store(pool: PgPool) {
    let store_id = unique_uuid();
    insert_store(&pool, &make_store(store_id, fixed_timestamp(1))).await;

    let repository = SqlxCategoryRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let first = make_category(
        unique_uuid(),
        store_id,
        "mains",
        0,
        MenuStatus::Active,
        fixed_timestamp(1_700_000_100),
    );
    let duplicate = make_category(
        unique_uuid(),
        store_id,
        "mains",
        1,
        MenuStatus::Active,
        fixed_timestamp(1_700_000_200),
    );

    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), &first).await.unwrap();
    let error = repository
        .insert(tx.as_mut(), &duplicate)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::Conflict { ref message } if message == "category slug already exists in store")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn sqlx_item_repository_rejects_duplicate_slug_within_store(pool: PgPool) {
    let store_id = unique_uuid();
    let category_a = unique_uuid();
    let category_b = unique_uuid();
    insert_store(&pool, &make_store(store_id, fixed_timestamp(1_700_000_000))).await;
    insert_category(
        &pool,
        &make_category(
            category_a,
            store_id,
            "mains",
            0,
            MenuStatus::Active,
            fixed_timestamp(1_700_000_100),
        ),
    )
    .await;
    insert_category(
        &pool,
        &make_category(
            category_b,
            store_id,
            "drinks",
            1,
            MenuStatus::Active,
            fixed_timestamp(1_700_000_200),
        ),
    )
    .await;

    let repository = SqlxItemRepository;
    let transactions = SqlxTransactionManager::new(pool);
    let first = make_item(
        unique_uuid(),
        store_id,
        category_a,
        "fried-rice",
        1800,
        0,
        MenuStatus::Active,
        fixed_timestamp(1_700_000_300),
    );
    let duplicate = make_item(
        unique_uuid(),
        store_id,
        category_b,
        "fried-rice",
        2000,
        1,
        MenuStatus::Active,
        fixed_timestamp(1_700_000_400),
    );

    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), &first).await.unwrap();
    let error = repository
        .insert(tx.as_mut(), &duplicate)
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::Conflict { ref message } if message == "item slug already exists in store")
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn migration_rejects_negative_price_amount(pool: PgPool) {
    let store_id = unique_uuid();
    let category_id = unique_uuid();
    insert_store(&pool, &make_store(store_id, fixed_timestamp(1_700_000_000))).await;
    insert_category(
        &pool,
        &make_category(
            category_id,
            store_id,
            "mains",
            0,
            MenuStatus::Active,
            fixed_timestamp(1_700_000_100),
        ),
    )
    .await;

    let error = sqlx::query(
        r#"
        INSERT INTO menu.items (
            id, store_id, category_id, slug, name, description, image_url,
            price_amount, sort_order, status, created_at, updated_at, deleted_at
        )
        VALUES ($1, $2, $3, 'bad-item', 'Bad Item', NULL, NULL, -1, 0, 'active', NOW(), NOW(), NULL)
        "#,
    )
    .bind(unique_uuid())
    .bind(store_id)
    .bind(category_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert!(error.as_database_error().is_some());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn migration_rejects_invalid_status_value(pool: PgPool) {
    let error = sqlx::query(
        r#"
        INSERT INTO menu.stores (
            id, slug, name, currency_code, timezone, status, created_at, updated_at, deleted_at
        )
        VALUES ($1, 'bad-store', 'Bad Store', 'CNY', 'Asia/Shanghai', 'archived', NOW(), NOW(), NULL)
        "#,
    )
    .bind(unique_uuid())
    .execute(&pool)
    .await
    .unwrap_err();

    assert!(error.as_database_error().is_some());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn read_repositories_return_only_active_non_deleted_records_in_sorted_order(pool: PgPool) {
    let store_id = unique_uuid();
    let category_fast = unique_uuid();
    let category_hidden = unique_uuid();
    let category_later = unique_uuid();
    insert_store(&pool, &make_store(store_id, fixed_timestamp(1_700_000_000))).await;

    insert_category(
        &pool,
        &make_category(
            category_later,
            store_id,
            "later",
            20,
            MenuStatus::Active,
            fixed_timestamp(2),
        ),
    )
    .await;
    insert_category(
        &pool,
        &make_category(
            category_fast,
            store_id,
            "fast",
            10,
            MenuStatus::Active,
            fixed_timestamp(3),
        ),
    )
    .await;
    insert_category(
        &pool,
        &make_category(
            category_hidden,
            store_id,
            "hidden",
            5,
            MenuStatus::Inactive,
            fixed_timestamp(4),
        ),
    )
    .await;

    let deleted_item_id = unique_uuid();
    let visible_item_a = unique_uuid();
    let visible_item_b = unique_uuid();
    let hidden_item_id = unique_uuid();
    insert_item(
        &pool,
        &make_item(
            visible_item_b,
            store_id,
            category_fast,
            "item-b",
            2200,
            20,
            MenuStatus::Active,
            fixed_timestamp(5),
        ),
    )
    .await;
    insert_item(
        &pool,
        &make_item(
            visible_item_a,
            store_id,
            category_fast,
            "item-a",
            1800,
            10,
            MenuStatus::Active,
            fixed_timestamp(6),
        ),
    )
    .await;
    insert_item(
        &pool,
        &make_item(
            hidden_item_id,
            store_id,
            category_fast,
            "hidden-item",
            1500,
            5,
            MenuStatus::Inactive,
            fixed_timestamp(7),
        ),
    )
    .await;
    insert_item(
        &pool,
        &make_item(
            deleted_item_id,
            store_id,
            category_later,
            "deleted-item",
            1300,
            1,
            MenuStatus::Active,
            fixed_timestamp(8),
        ),
    )
    .await;
    sqlx::query("UPDATE menu.items SET deleted_at = NOW() WHERE id = $1")
        .bind(deleted_item_id)
        .execute(&pool)
        .await
        .unwrap();

    let store_read_repository = SqlxStoreReadRepository::new(pool.clone());
    let category_read_repository = SqlxCategoryReadRepository::new(pool.clone());
    let item_read_repository = SqlxItemReadRepository::new(pool.clone());
    let store = store_read_repository.get_active().await.unwrap().unwrap();
    let categories = category_read_repository
        .list_active_by_store(&StoreId::new(store_id.to_string()))
        .await
        .unwrap();
    let items = item_read_repository
        .list_active_by_store(
            &StoreId::new(store_id.to_string()),
            ItemListFilter {
                category_id: Some(CategoryId::new(category_fast.to_string())),
            },
        )
        .await
        .unwrap();

    assert_eq!(store.status, "active");
    assert_eq!(
        categories
            .iter()
            .map(|category| category.slug.as_str())
            .collect::<Vec<_>>(),
        vec!["fast", "later"]
    );
    assert_eq!(
        items
            .iter()
            .map(|item| item.slug.as_str())
            .collect::<Vec<_>>(),
        vec!["item-a", "item-b"]
    );

    let hidden_item = item_read_repository
        .get_active_by_id(&ItemId::new(hidden_item_id.to_string()))
        .await
        .unwrap();
    let deleted_item = item_read_repository
        .get_active_by_id(&ItemId::new(deleted_item_id.to_string()))
        .await
        .unwrap();
    let visible_item = item_read_repository
        .get_active_by_id(&ItemId::new(visible_item_a.to_string()))
        .await
        .unwrap()
        .unwrap();

    assert!(hidden_item.is_none());
    assert!(deleted_item.is_none());
    assert_eq!(visible_item.price_amount, 1800);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn migration_creates_expected_schema_tables_constraints_and_indexes(pool: PgPool) {
    let store_columns: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns WHERE table_schema = 'menu' AND table_name = 'stores'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let category_unique: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_constraint WHERE connamespace = 'menu'::regnamespace AND conname = 'categories_store_slug_unique'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let item_index_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_indexes WHERE schemaname = 'menu' AND indexname = 'idx_menu_items_store_category_status_sort'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(store_columns, 9);
    assert_eq!(category_unique, 1);
    assert_eq!(item_index_count, 1);
}
