use ordering_food_fulfillment_application::{
    CommercialOrderProjectionReadModel, CommercialOrderProjectionReadRepository,
    CommercialOrderProjectionStore, TransactionManager,
};
use ordering_food_fulfillment_infrastructure_sqlx::{
    SqlxCommercialOrderProjectionRepository, SqlxOutboxMessageRepository,
    SqlxProjectionCheckpointStore, SqlxTransactionManager,
};
use sqlx::PgPool;
use time::macros::datetime;
use uuid::Uuid;

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn commercial_order_projection_round_trips(pool: PgPool) {
    let repository = SqlxCommercialOrderProjectionRepository::new(pool.clone());
    let transaction_manager = SqlxTransactionManager::new(pool);
    let mut tx = transaction_manager.begin().await.unwrap();
    let order_id = Uuid::now_v7().to_string();
    let customer_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();
    let item_id = Uuid::now_v7().to_string();

    repository
        .upsert(
            tx.as_mut(),
            &CommercialOrderProjectionReadModel {
                order_id: order_id.clone(),
                customer_id,
                store_id,
                status: "placed".to_string(),
                subtotal_amount: 1800,
                total_amount: 1800,
                created_at: datetime!(2026-03-15 10:00 UTC),
                updated_at: datetime!(2026-03-15 10:00 UTC),
                items: vec![
                    ordering_food_fulfillment_application::CommercialOrderProjectionItemReadModel {
                        line_number: 1,
                        catalog_item_id: item_id,
                        name: "Noodles".to_string(),
                        unit_price_amount: 1800,
                        quantity: 1,
                        line_total_amount: 1800,
                    },
                ],
            },
        )
        .await
        .unwrap();
    transaction_manager.commit(tx).await.unwrap();

    let projection = repository
        .get_by_ordering_order_id(&order_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(projection.status, "placed");
    assert_eq!(projection.items.len(), 1);
    assert_eq!(projection.items[0].line_number, 1);
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn checkpoint_store_defaults_and_updates(pool: PgPool) {
    let store = SqlxProjectionCheckpointStore::new(pool);
    let checkpoint = store.get("fulfillment.ordering-commercial").await.unwrap();
    assert_eq!(checkpoint.last_processed_id, 0);

    store
        .save(
            "fulfillment.ordering-commercial",
            42,
            datetime!(2026-03-15 10:30 UTC),
        )
        .await
        .unwrap();

    let checkpoint = store.get("fulfillment.ordering-commercial").await.unwrap();
    assert_eq!(checkpoint.last_processed_id, 42);
    assert_eq!(checkpoint.updated_at, datetime!(2026-03-15 10:30 UTC));
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn outbox_repository_reads_ordering_messages_in_id_order(pool: PgPool) {
    sqlx::query(
        r#"
        INSERT INTO platform.outbox_messages (
            producer_context,
            event_type,
            aggregate_id,
            payload,
            occurred_at,
            available_at,
            created_at
        )
        VALUES
            ('ordering', 'ordering.order_placed', 'order-1', '{}'::jsonb, $1, $1, $1),
            ('ordering', 'ordering.order_cancelled_by_customer', 'order-1', '{}'::jsonb, $2, $2, $2)
        "#,
    )
    .bind(datetime!(2026-03-15 10:00 UTC))
    .bind(datetime!(2026-03-15 10:05 UTC))
    .execute(&pool)
    .await
    .unwrap();

    let repository = SqlxOutboxMessageRepository::new(pool);
    let messages = repository
        .list_available("ordering", 0, datetime!(2026-03-15 11:00 UTC), 50)
        .await
        .unwrap();

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].event_type, "ordering.order_placed");
    assert_eq!(
        messages[1].event_type,
        "ordering.order_cancelled_by_customer"
    );
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn outbox_repository_hides_failed_messages_from_available_batch(pool: PgPool) {
    let first_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO platform.outbox_messages (
            producer_context,
            event_type,
            aggregate_id,
            payload,
            occurred_at,
            available_at,
            created_at
        )
        VALUES ('ordering', 'ordering.order_placed', 'order-1', '{}'::jsonb, $1, $1, $1)
        RETURNING id
        "#,
    )
    .bind(datetime!(2026-03-15 10:00 UTC))
    .fetch_one(&pool)
    .await
    .unwrap();
    let second_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO platform.outbox_messages (
            producer_context,
            event_type,
            aggregate_id,
            payload,
            occurred_at,
            available_at,
            created_at
        )
        VALUES (
            'ordering',
            'ordering.order_cancelled_by_customer',
            'order-2',
            '{}'::jsonb,
            $1,
            $1,
            $1
        )
        RETURNING id
        "#,
    )
    .bind(datetime!(2026-03-15 10:05 UTC))
    .fetch_one(&pool)
    .await
    .unwrap();

    let repository = SqlxOutboxMessageRepository::new(pool);
    repository
        .record_failure(first_id, "decode failed")
        .await
        .unwrap();

    let messages = repository
        .list_available("ordering", 0, datetime!(2026-03-15 11:00 UTC), 50)
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, second_id);
    assert_eq!(
        messages[0].event_type,
        "ordering.order_cancelled_by_customer"
    );
}
