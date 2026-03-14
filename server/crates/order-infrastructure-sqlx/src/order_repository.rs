use crate::{db_order_status::DbOrderStatus, transaction::SqlxTransactionContext};
use async_trait::async_trait;
use ordering_food_order_application::{ApplicationError, OrderRepository, TransactionContext};
use ordering_food_order_domain::{
    CustomerId, MenuItemId, Order, OrderId, OrderItem, OrderStatus, StoreId,
};
use ordering_food_shared_kernel::Identifier;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct SqlxOrderRepository;

impl SqlxOrderRepository {
    fn transaction(
        tx: &mut dyn TransactionContext,
    ) -> Result<&mut Transaction<'static, Postgres>, ApplicationError> {
        tx.as_any_mut()
            .downcast_mut::<SqlxTransactionContext>()
            .map(SqlxTransactionContext::transaction_mut)
            .ok_or_else(|| {
                ApplicationError::unexpected("unexpected transaction context implementation")
            })
    }

    fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, ApplicationError> {
        Uuid::parse_str(value)
            .map_err(|_| ApplicationError::validation(format!("{field} must be a valid UUID")))
    }

    async fn load_items(
        transaction: &mut Transaction<'static, Postgres>,
        order_id: Uuid,
    ) -> Result<Vec<OrderItem>, ApplicationError> {
        let rows = sqlx::query(
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
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to load order items", error)
        })?;

        rows.into_iter()
            .map(|row| {
                OrderItem::rehydrate(
                    row.get("line_number"),
                    MenuItemId::new(row.get::<Uuid, _>("menu_item_id").to_string()),
                    row.get::<String, _>("name"),
                    row.get("unit_price_amount"),
                    row.get("quantity"),
                    row.get("line_total_amount"),
                )
                .map_err(ApplicationError::from)
            })
            .collect()
    }
}

#[async_trait]
impl OrderRepository for SqlxOrderRepository {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        order_id: &OrderId,
    ) -> Result<Option<Order>, ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let order_id = Self::parse_uuid(order_id.as_str(), "order id")?;

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
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to load order aggregate", error)
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        let items = Self::load_items(transaction, order_id).await?;
        let order = Order::rehydrate(
            OrderId::new(row.get::<Uuid, _>("id").to_string()),
            CustomerId::new(row.get::<Uuid, _>("customer_id").to_string()),
            StoreId::new(row.get::<Uuid, _>("store_id").to_string()),
            OrderStatus::from(row.get::<DbOrderStatus, _>("status")),
            items,
            row.get("subtotal_amount"),
            row.get("total_amount"),
            row.get("created_at"),
            row.get("updated_at"),
        )?;

        Ok(Some(order))
    }

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        order: &Order,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let order_id = Self::parse_uuid(order.id().as_str(), "order id")?;
        let customer_id = Self::parse_uuid(order.customer_id().as_str(), "customer id")?;
        let store_id = Self::parse_uuid(order.store_id().as_str(), "store id")?;

        sqlx::query(
            r#"
            INSERT INTO ordering.orders (
                id,
                customer_id,
                store_id,
                status,
                subtotal_amount,
                total_amount,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(order_id)
        .bind(customer_id)
        .bind(store_id)
        .bind(DbOrderStatus::from(order.status()))
        .bind(order.subtotal_amount())
        .bind(order.total_amount())
        .bind(order.created_at())
        .bind(order.updated_at())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to insert order", error)
        })?;

        for item in order.items() {
            sqlx::query(
                r#"
                INSERT INTO ordering.order_items (
                    order_id,
                    line_number,
                    menu_item_id,
                    name,
                    unit_price_amount,
                    quantity,
                    line_total_amount
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(order_id)
            .bind(item.line_number())
            .bind(Self::parse_uuid(
                item.menu_item_id().as_str(),
                "menu item id",
            )?)
            .bind(item.name())
            .bind(item.unit_price_amount())
            .bind(item.quantity())
            .bind(item.line_total_amount())
            .execute(&mut **transaction)
            .await
            .map_err(|error| {
                ApplicationError::unexpected_with_source("failed to insert order item", error)
            })?;
        }

        Ok(())
    }

    async fn update(
        &self,
        tx: &mut dyn TransactionContext,
        order: &Order,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let order_id = Self::parse_uuid(order.id().as_str(), "order id")?;

        sqlx::query(
            r#"
            UPDATE ordering.orders
            SET status = $2, subtotal_amount = $3, total_amount = $4, updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(order_id)
        .bind(DbOrderStatus::from(order.status()))
        .bind(order.subtotal_amount())
        .bind(order.total_amount())
        .bind(order.updated_at())
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to update order", error)
        })?;

        Ok(())
    }
}
