use ordering_food_fulfillment_application::Clock;
use ordering_food_fulfillment_infrastructure_sqlx::build_fulfillment_module;
use ordering_food_fulfillment_integration::build_ordering_event_projector;
use sqlx::PgPool;
use time::{OffsetDateTime, macros::datetime};
use uuid::Uuid;

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        datetime!(2026-03-15 10:00 UTC)
    }
}

async fn insert_outbox_message(
    pool: &PgPool,
    event_type: &str,
    aggregate_id: &str,
    payload: serde_json::Value,
    occurred_at: OffsetDateTime,
) {
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
        VALUES ($1, $2, $3, $4, $5, $5, $5)
        "#,
    )
    .bind("ordering")
    .bind(event_type)
    .bind(aggregate_id)
    .bind(payload)
    .bind(occurred_at)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
#[ignore = "requires DATABASE_URL"]
async fn projector_bootstraps_local_projection_and_workflow(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock);
    let module = build_fulfillment_module(pool.clone(), clock.clone());
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let order_id = Uuid::now_v7().to_string();
    let customer_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();
    let item_id = Uuid::now_v7().to_string();

    insert_outbox_message(
        &pool,
        "ordering.order_placed",
        &order_id,
        serde_json::json!({
            "order_id": order_id,
            "customer_id": customer_id,
            "store_id": store_id,
            "status": "placed",
            "subtotal_amount": 1800,
            "total_amount": 1800,
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "items": [{
                "line_number": 1,
                "catalog_item_id": item_id,
                "name": "Noodles",
                "unit_price_amount": 1800,
                "quantity": 1,
                "line_total_amount": 1800
            }]
        }),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;

    let result = projector.project_once().await.unwrap();
    assert_eq!(result.applied_count, 1);

    let projection = module
        .commercial_queries()
        .get_by_ordering_order_id(&order_id)
        .await
        .unwrap()
        .unwrap();
    let workflow = module
        .workflow_queries()
        .get_by_ordering_order_id(&order_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(projection.status, "placed");
    assert_eq!(projection.items.len(), 1);
    assert_eq!(workflow.status, "pending_acceptance");
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
#[ignore = "requires DATABASE_URL"]
async fn projector_applies_customer_cancellation_to_local_models(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock);
    let module = build_fulfillment_module(pool.clone(), clock.clone());
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let order_id = Uuid::now_v7().to_string();
    let customer_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();

    insert_outbox_message(
        &pool,
        "ordering.order_placed",
        &order_id,
        serde_json::json!({
            "order_id": order_id,
            "customer_id": customer_id,
            "store_id": store_id,
            "status": "placed",
            "subtotal_amount": 1800,
            "total_amount": 1800,
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "items": []
        }),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;
    insert_outbox_message(
        &pool,
        "ordering.order_cancelled_by_customer",
        &order_id,
        serde_json::json!({
            "order_id": order_id,
            "customer_id": customer_id,
            "store_id": store_id,
            "occurred_at": "2026-03-15T10:15:00Z"
        }),
        datetime!(2026-03-15 10:15 UTC),
    )
    .await;

    let result = projector.project_once().await.unwrap();
    assert_eq!(result.applied_count, 2);

    let projection = module
        .commercial_queries()
        .get_by_ordering_order_id(&order_id)
        .await
        .unwrap()
        .unwrap();
    let workflow = module
        .workflow_queries()
        .get_by_ordering_order_id(&order_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(projection.status, "cancelled_by_customer");
    assert_eq!(workflow.status, "cancelled_by_customer");
}
