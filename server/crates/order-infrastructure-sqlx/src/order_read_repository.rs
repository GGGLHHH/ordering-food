use crate::db_order_status::DbOrderStatus;
use async_trait::async_trait;
use ordering_food_order_application::{
    ApplicationError, OrderItemReadModel, OrderListItemReadModel, OrderReadModel,
    OrderReadRepository,
};
use ordering_food_order_domain::OrderId;
use ordering_food_shared_kernel::Identifier;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxOrderReadRepository {
    pool: PgPool,
}

impl SqlxOrderReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn parse_order_id(order_id: &OrderId) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(order_id.as_str())
            .map_err(|_| ApplicationError::validation("order id must be a valid UUID"))
    }
}

#[async_trait]
impl OrderReadRepository for SqlxOrderReadRepository {
    async fn get_by_id(
        &self,
        order_id: &OrderId,
    ) -> Result<Option<OrderReadModel>, ApplicationError> {
        let order_id = Self::parse_order_id(order_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                customer_id,
                store_id,
                status,
                subtotal_amount,
                total_amount,
                created_at,
                updated_at
            FROM ordering.orders
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query order", error)
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        let item_rows = sqlx::query(
            r#"
            SELECT
                line_number,
                menu_item_id,
                name,
                unit_price_amount,
                quantity,
                line_total_amount
            FROM ordering.order_items
            WHERE order_id = $1
            ORDER BY line_number ASC
            "#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query order items", error)
        })?;

        Ok(Some(OrderReadModel {
            order_id: row.get::<Uuid, _>("id").to_string(),
            customer_id: row.get::<Uuid, _>("customer_id").to_string(),
            store_id: row.get::<Uuid, _>("store_id").to_string(),
            status: DbOrderStatus::to_status_string(row.get::<DbOrderStatus, _>("status")),
            subtotal_amount: row.get("subtotal_amount"),
            total_amount: row.get("total_amount"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            items: item_rows
                .into_iter()
                .map(|row| OrderItemReadModel {
                    line_number: row.get("line_number"),
                    menu_item_id: row.get::<Uuid, _>("menu_item_id").to_string(),
                    name: row.get("name"),
                    unit_price_amount: row.get("unit_price_amount"),
                    quantity: row.get("quantity"),
                    line_total_amount: row.get("line_total_amount"),
                })
                .collect(),
        }))
    }

    async fn list_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<OrderListItemReadModel>, ApplicationError> {
        let customer_id = Uuid::parse_str(customer_id)
            .map_err(|_| ApplicationError::validation("customer id must be a valid UUID"))?;

        let rows = sqlx::query(
            r#"
            SELECT
                o.id,
                o.customer_id,
                o.store_id,
                o.status,
                o.subtotal_amount,
                o.total_amount,
                o.created_at,
                o.updated_at,
                COUNT(oi.line_number) AS item_count
            FROM ordering.orders o
            LEFT JOIN ordering.order_items oi ON oi.order_id = o.id
            WHERE o.customer_id = $1
            GROUP BY
                o.id,
                o.customer_id,
                o.store_id,
                o.status,
                o.subtotal_amount,
                o.total_amount,
                o.created_at,
                o.updated_at
            ORDER BY o.created_at DESC, o.id DESC
            "#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to query customer orders", error)
        })?;

        Ok(rows
            .into_iter()
            .map(|row| OrderListItemReadModel {
                order_id: row.get::<Uuid, _>("id").to_string(),
                customer_id: row.get::<Uuid, _>("customer_id").to_string(),
                store_id: row.get::<Uuid, _>("store_id").to_string(),
                status: DbOrderStatus::to_status_string(row.get::<DbOrderStatus, _>("status")),
                subtotal_amount: row.get("subtotal_amount"),
                total_amount: row.get("total_amount"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                item_count: row.get::<i64, _>("item_count") as usize,
            })
            .collect())
    }
}

impl DbOrderStatus {
    fn to_status_string(value: Self) -> String {
        match value {
            Self::PendingAcceptance => "pending_acceptance",
            Self::Accepted => "accepted",
            Self::Preparing => "preparing",
            Self::ReadyForPickup => "ready_for_pickup",
            Self::Completed => "completed",
            Self::CancelledByCustomer => "cancelled_by_customer",
            Self::RejectedByStore => "rejected_by_store",
        }
        .to_string()
    }
}
