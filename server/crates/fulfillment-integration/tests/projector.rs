use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    ApplicationError, Clock, WorkflowAction, WorkflowActionAuthorizer,
};
use ordering_food_fulfillment_infrastructure_sqlx::build_fulfillment_module;
use ordering_food_fulfillment_integration::build_ordering_event_projector;
use ordering_food_ordering_published::{
    COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE, COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
    CommercialOrderCancelledByCustomerV1, CommercialOrderLineSnapshotV1, CommercialOrderPlacedV1,
};
use sqlx::PgPool;
use time::{OffsetDateTime, macros::datetime};
use uuid::Uuid;

#[test]
fn fulfillment_application_boundary_stays_transport_free() {
    let integration_manifest = include_str!("../Cargo.toml");
    let projector_source = include_str!("../src/projector.rs");
    let application_manifest = include_str!("../../fulfillment-application/Cargo.toml");
    let ports_source = include_str!("../../fulfillment-application/src/ports.rs");
    let lib_source = include_str!("../../fulfillment-application/src/lib.rs");
    let handler_source =
        include_str!("../../fulfillment-application/src/ordering_event_handler.rs");

    assert!(integration_manifest.contains("ordering-food-ordering-published"));
    assert!(projector_source.contains("CommercialOrderPlacedV1"));
    assert!(projector_source.contains("CommercialOrderStatusChangedV1"));
    assert!(!application_manifest.contains("ordering-food-ordering-published"));
    assert!(!application_manifest.contains("serde_json.workspace"));
    assert!(!ports_source.contains("ordering_food_ordering_published"));
    assert!(!ports_source.contains("OutboxMessage"));
    assert!(!ports_source.contains("OutboxMessageReader"));
    assert!(!ports_source.contains("ProjectionCheckpoint"));
    assert!(!ports_source.contains("ProjectionCheckpointStore"));
    assert!(!ports_source.contains("serde_json::Value"));
    assert!(!lib_source.contains("OutboxMessage"));
    assert!(!lib_source.contains("ProjectionCheckpoint"));
    assert!(!handler_source.contains("ordering_food_ordering_published"));
}

struct NoopWorkflowActionAuthorizer;

#[async_trait]
impl WorkflowActionAuthorizer for NoopWorkflowActionAuthorizer {
    async fn ensure_actor_can_manage_order(
        &self,
        _actor_user_id: &str,
        _store_id: &str,
        _action: WorkflowAction,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FixedClock {
    now: OffsetDateTime,
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.now
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

async fn insert_outbox_message_returning_id(
    pool: &PgPool,
    event_type: &str,
    aggregate_id: &str,
    payload: serde_json::Value,
    occurred_at: OffsetDateTime,
) -> i64 {
    let row = sqlx::query(
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
        RETURNING id
        "#,
    )
    .bind("ordering")
    .bind(event_type)
    .bind(aggregate_id)
    .bind(payload)
    .bind(occurred_at)
    .fetch_one(pool)
    .await
    .unwrap();

    sqlx::Row::get(&row, "id")
}

async fn load_checkpoint_last_processed_id(pool: &PgPool) -> Option<i64> {
    sqlx::query(
        r#"
        SELECT last_processed_id
        FROM platform.projection_checkpoints
        WHERE projector_name = 'fulfillment.ordering-commercial'
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .unwrap()
    .map(|row| sqlx::Row::get(&row, "last_processed_id"))
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn projector_bootstraps_local_projection_and_workflow(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:00 UTC),
    });
    let authorizer: std::sync::Arc<dyn WorkflowActionAuthorizer> =
        std::sync::Arc::new(NoopWorkflowActionAuthorizer);
    let module = build_fulfillment_module(pool.clone(), clock.clone(), authorizer);
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let order_id = Uuid::now_v7().to_string();
    let customer_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();
    let item_id = Uuid::now_v7().to_string();

    insert_outbox_message(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &order_id,
        serde_json::to_value(CommercialOrderPlacedV1 {
            order_id: order_id.clone(),
            customer_id: customer_id.clone(),
            store_id: store_id.clone(),
            subtotal_amount: 1800,
            total_amount: 1800,
            occurred_at: datetime!(2026-03-15 10:00 UTC),
            items: vec![CommercialOrderLineSnapshotV1 {
                line_number: 1,
                catalog_item_id: item_id,
                name: "Noodles".to_string(),
                unit_price_amount: 1800,
                quantity: 1,
                line_total_amount: 1800,
            }],
        })
        .unwrap(),
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
async fn projector_applies_customer_cancellation_to_local_models(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:15 UTC),
    });
    let authorizer: std::sync::Arc<dyn WorkflowActionAuthorizer> =
        std::sync::Arc::new(NoopWorkflowActionAuthorizer);
    let module = build_fulfillment_module(pool.clone(), clock.clone(), authorizer);
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let order_id = Uuid::now_v7().to_string();
    let customer_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();

    insert_outbox_message(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &order_id,
        serde_json::to_value(CommercialOrderPlacedV1 {
            order_id: order_id.clone(),
            customer_id: customer_id.clone(),
            store_id: store_id.clone(),
            subtotal_amount: 1800,
            total_amount: 1800,
            occurred_at: datetime!(2026-03-15 10:00 UTC),
            items: vec![],
        })
        .unwrap(),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;
    insert_outbox_message(
        &pool,
        COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE,
        &order_id,
        serde_json::to_value(CommercialOrderCancelledByCustomerV1 {
            order_id: order_id.clone(),
            customer_id: customer_id,
            store_id: store_id.clone(),
            occurred_at: datetime!(2026-03-15 10:15 UTC),
        })
        .unwrap(),
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

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn projector_records_decode_failure_without_advancing_checkpoint(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:00 UTC),
    });
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let order_id = Uuid::now_v7().to_string();

    insert_outbox_message(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &order_id,
        serde_json::json!({
            "order_id": order_id
        }),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;

    let error = projector.project_once().await.unwrap_err();
    assert!(matches!(
        error,
        ordering_food_fulfillment_integration::OrderingEventProjectorError::Decode { .. }
    ));

    let row = sqlx::query(
        r#"
        SELECT error_count, last_error
        FROM platform.outbox_messages
        WHERE aggregate_id = $1
        ORDER BY id DESC
        LIMIT 1
        "#,
    )
    .bind(&order_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let error_count: i32 = sqlx::Row::get(&row, "error_count");
    let last_error: Option<String> = sqlx::Row::get(&row, "last_error");
    assert_eq!(error_count, 1);
    assert!(last_error.is_some());

    let checkpoint_row = sqlx::query(
        r#"
        SELECT last_processed_id
        FROM platform.projection_checkpoints
        WHERE projector_name = 'fulfillment.ordering-commercial'
        LIMIT 1
        "#,
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(checkpoint_row.is_none());
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn projector_advances_checkpoint_after_successful_batch(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:05 UTC),
    });
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let first_order_id = Uuid::now_v7().to_string();
    let second_order_id = Uuid::now_v7().to_string();
    let customer_id = Uuid::now_v7().to_string();
    let store_id = Uuid::now_v7().to_string();

    insert_outbox_message_returning_id(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &first_order_id,
        serde_json::to_value(CommercialOrderPlacedV1 {
            order_id: first_order_id.clone(),
            customer_id: customer_id.clone(),
            store_id: store_id.clone(),
            subtotal_amount: 1800,
            total_amount: 1800,
            occurred_at: datetime!(2026-03-15 10:00 UTC),
            items: vec![],
        })
        .unwrap(),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;
    let second_id = insert_outbox_message_returning_id(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &second_order_id,
        serde_json::to_value(CommercialOrderPlacedV1 {
            order_id: second_order_id.clone(),
            customer_id,
            store_id,
            subtotal_amount: 2000,
            total_amount: 2000,
            occurred_at: datetime!(2026-03-15 10:05 UTC),
            items: vec![],
        })
        .unwrap(),
        datetime!(2026-03-15 10:05 UTC),
    )
    .await;

    let result = projector.project_once().await.unwrap();

    assert_eq!(result.scanned_count, 2);
    assert_eq!(result.applied_count, 2);
    assert_eq!(result.skipped_count, 0);
    assert_eq!(result.last_processed_id, second_id);
    assert_eq!(load_checkpoint_last_processed_id(&pool).await, Some(second_id));
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn projector_skips_unknown_event_and_advances_checkpoint(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:00 UTC),
    });
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let order_id = Uuid::now_v7().to_string();

    let skipped_id = insert_outbox_message_returning_id(
        &pool,
        "ordering.unknown_event",
        &order_id,
        serde_json::json!({ "order_id": order_id }),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;

    let result = projector.project_once().await.unwrap();

    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.applied_count, 0);
    assert_eq!(result.skipped_count, 1);
    assert_eq!(result.last_processed_id, skipped_id);
    assert_eq!(load_checkpoint_last_processed_id(&pool).await, Some(skipped_id));
}

#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn projector_reaches_later_messages_after_decode_failure(pool: PgPool) {
    let clock = std::sync::Arc::new(FixedClock {
        now: datetime!(2026-03-15 10:10 UTC),
    });
    let authorizer: std::sync::Arc<dyn WorkflowActionAuthorizer> =
        std::sync::Arc::new(NoopWorkflowActionAuthorizer);
    let module = build_fulfillment_module(pool.clone(), clock.clone(), authorizer);
    let projector = build_ordering_event_projector(pool.clone(), clock);
    let broken_order_id = Uuid::now_v7().to_string();

    let broken_id = insert_outbox_message_returning_id(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &broken_order_id,
        serde_json::json!({
            "order_id": broken_order_id
        }),
        datetime!(2026-03-15 10:00 UTC),
    )
    .await;

    let error = projector.project_once().await.unwrap_err();
    assert!(matches!(
        error,
        ordering_food_fulfillment_integration::OrderingEventProjectorError::Decode { .. }
    ));
    assert_eq!(load_checkpoint_last_processed_id(&pool).await, None);

    let recovered_order_id = Uuid::now_v7().to_string();
    let recovered_customer_id = Uuid::now_v7().to_string();
    let recovered_store_id = Uuid::now_v7().to_string();
    let recovered_id = insert_outbox_message_returning_id(
        &pool,
        COMMERCIAL_ORDER_PLACED_EVENT_TYPE,
        &recovered_order_id,
        serde_json::to_value(CommercialOrderPlacedV1 {
            order_id: recovered_order_id.clone(),
            customer_id: recovered_customer_id.clone(),
            store_id: recovered_store_id.clone(),
            subtotal_amount: 1800,
            total_amount: 1800,
            occurred_at: datetime!(2026-03-15 10:05 UTC),
            items: vec![],
        })
        .unwrap(),
        datetime!(2026-03-15 10:05 UTC),
    )
    .await;

    let result = projector.project_once().await.unwrap();

    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.applied_count, 1);
    assert_eq!(result.skipped_count, 0);
    assert_eq!(result.last_processed_id, recovered_id);
    assert_eq!(
        load_checkpoint_last_processed_id(&pool).await,
        Some(recovered_id)
    );

    let failed_row = sqlx::query(
        r#"
        SELECT error_count
        FROM platform.outbox_messages
        WHERE id = $1
        LIMIT 1
        "#,
    )
    .bind(broken_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let error_count: i32 = sqlx::Row::get(&failed_row, "error_count");
    assert_eq!(error_count, 1);

    let projection = module
        .commercial_queries()
        .get_by_ordering_order_id(&recovered_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(projection.customer_id, recovered_customer_id);
    assert_eq!(projection.store_id, recovered_store_id);
    assert_eq!(projection.status, "placed");
}
