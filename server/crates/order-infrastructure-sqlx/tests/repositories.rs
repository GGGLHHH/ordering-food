use ordering_food_order_application::{OrderReadRepository, OrderRepository, TransactionManager};
use ordering_food_order_domain::{
    CustomerId, MenuItemId, Order, OrderId, OrderStatus, PlaceOrderItemInput, StoreId,
};
use ordering_food_order_infrastructure_sqlx::{
    SqlxOrderReadRepository, SqlxOrderRepository, SqlxTransactionManager,
};
use ordering_food_shared_kernel::Identifier;
use sqlx::PgPool;
use time::macros::datetime;
use uuid::Uuid;

async fn insert_order(pool: &PgPool, order: &Order) {
    let repository = SqlxOrderRepository;
    let transactions = SqlxTransactionManager::new(pool.clone());
    let mut tx = transactions.begin().await.unwrap();
    repository.insert(tx.as_mut(), order).await.unwrap();
    transactions.commit(tx).await.unwrap();
}

fn make_order(order_id: Uuid, customer_id: Uuid, store_id: Uuid) -> Order {
    Order::place(
        OrderId::new(order_id.to_string()),
        CustomerId::new(customer_id.to_string()),
        StoreId::new(store_id.to_string()),
        vec![
            PlaceOrderItemInput {
                menu_item_id: MenuItemId::new(Uuid::now_v7().to_string()),
                name: "Braised Pork Rice".to_string(),
                unit_price_amount: 2800,
                quantity: 1,
            },
            PlaceOrderItemInput {
                menu_item_id: MenuItemId::new(Uuid::now_v7().to_string()),
                name: "Iced Lemon Tea".to_string(),
                unit_price_amount: 900,
                quantity: 2,
            },
        ],
        datetime!(2026-03-15 10:00 UTC),
    )
    .unwrap()
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn sqlx_order_repositories_persist_and_load_snapshot(pool: PgPool) {
    let order = make_order(Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    insert_order(&pool, &order).await;

    let query_repository = SqlxOrderReadRepository::new(pool.clone());
    let read_model = query_repository
        .get_by_id(order.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(read_model.order_id, order.id().as_str());
    assert_eq!(read_model.customer_id, order.customer_id().as_str());
    assert_eq!(read_model.store_id, order.store_id().as_str());
    assert_eq!(read_model.status, "pending_acceptance");
    assert_eq!(read_model.items.len(), 2);
    assert_eq!(read_model.items[0].name, "Braised Pork Rice");
    assert_eq!(read_model.items[0].line_total_amount, 2800);
    assert_eq!(read_model.items[1].name, "Iced Lemon Tea");
    assert_eq!(read_model.items[1].line_total_amount, 1800);
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn postgres_enum_round_trips_with_domain_status(pool: PgPool) {
    let status: String = sqlx::query_scalar(
        r#"
        SELECT ($1::ordering.order_status)::text
        "#,
    )
    .bind("accepted")
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(OrderStatus::parse(status).unwrap(), OrderStatus::Accepted);
}
