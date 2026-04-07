use crate::transaction::SqlxTransactionContext;
use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    ApplicationError, CommercialOrderProjectionItemReadModel, CommercialOrderProjectionReadModel,
    CommercialOrderProjectionReadRepository, CommercialOrderProjectionStore, TransactionContext,
};
use ordering_food_shared_kernel::Timestamp;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxCommercialOrderProjectionRepository {
    pool: PgPool,
}

impl SqlxCommercialOrderProjectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

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
}

#[async_trait]
impl CommercialOrderProjectionReadRepository for SqlxCommercialOrderProjectionRepository {
    async fn get_by_ordering_order_id(
        &self,
        ordering_order_id: &str,
    ) -> Result<Option<CommercialOrderProjectionReadModel>, ApplicationError> {
        let ordering_order_id = Self::parse_uuid(ordering_order_id, "ordering order id")?;

        let row = sqlx::query(
            r#"
            SELECT
                ordering_order_id,
                customer_id,
                store_id,
                status,
                subtotal_amount,
                total_amount,
                created_at,
                updated_at
            FROM fulfillment.ordering_order_projections
            WHERE ordering_order_id = $1
            LIMIT 1
            "#,
        )
        .bind(ordering_order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query commercial order projection",
                error,
            )
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        let item_rows = sqlx::query(
            r#"
            SELECT
                line_number,
                catalog_item_id,
                name,
                unit_price_amount,
                quantity,
                line_total_amount
            FROM fulfillment.ordering_order_projection_items
            WHERE ordering_order_id = $1
            ORDER BY line_number ASC
            "#,
        )
        .bind(ordering_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to query commercial order projection items",
                error,
            )
        })?;

        Ok(Some(CommercialOrderProjectionReadModel {
            order_id: row.get::<Uuid, _>("ordering_order_id").to_string(),
            customer_id: row.get::<Uuid, _>("customer_id").to_string(),
            store_id: row.get::<Uuid, _>("store_id").to_string(),
            status: row.get("status"),
            subtotal_amount: row.get("subtotal_amount"),
            total_amount: row.get("total_amount"),
            created_at: row.get::<Timestamp, _>("created_at"),
            updated_at: row.get::<Timestamp, _>("updated_at"),
            items: item_rows
                .into_iter()
                .map(|item| CommercialOrderProjectionItemReadModel {
                    line_number: item.get("line_number"),
                    catalog_item_id: item.get::<Uuid, _>("catalog_item_id").to_string(),
                    name: item.get("name"),
                    unit_price_amount: item.get("unit_price_amount"),
                    quantity: item.get("quantity"),
                    line_total_amount: item.get("line_total_amount"),
                })
                .collect(),
        }))
    }
}

#[async_trait]
impl CommercialOrderProjectionStore for SqlxCommercialOrderProjectionRepository {
    async fn upsert(
        &self,
        tx: &mut dyn TransactionContext,
        projection: &CommercialOrderProjectionReadModel,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let order_id = Self::parse_uuid(&projection.order_id, "ordering order id")?;
        let customer_id = Self::parse_uuid(&projection.customer_id, "customer id")?;
        let store_id = Self::parse_uuid(&projection.store_id, "store id")?;

        sqlx::query(
            r#"
            INSERT INTO fulfillment.ordering_order_projections (
                ordering_order_id,
                customer_id,
                store_id,
                status,
                subtotal_amount,
                total_amount,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (ordering_order_id) DO UPDATE
            SET
                customer_id = EXCLUDED.customer_id,
                store_id = EXCLUDED.store_id,
                status = EXCLUDED.status,
                subtotal_amount = EXCLUDED.subtotal_amount,
                total_amount = EXCLUDED.total_amount,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(order_id)
        .bind(customer_id)
        .bind(store_id)
        .bind(&projection.status)
        .bind(projection.subtotal_amount)
        .bind(projection.total_amount)
        .bind(projection.created_at)
        .bind(projection.updated_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to upsert commercial order projection",
                error,
            )
        })?;

        sqlx::query(
            r#"
            DELETE FROM fulfillment.ordering_order_projection_items
            WHERE ordering_order_id = $1
            "#,
        )
        .bind(order_id)
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to replace commercial order projection items",
                error,
            )
        })?;

        for item in &projection.items {
            let catalog_item_id = Self::parse_uuid(&item.catalog_item_id, "catalog item id")?;
            sqlx::query(
                r#"
                INSERT INTO fulfillment.ordering_order_projection_items (
                    ordering_order_id,
                    line_number,
                    catalog_item_id,
                    name,
                    unit_price_amount,
                    quantity,
                    line_total_amount
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(order_id)
            .bind(item.line_number)
            .bind(catalog_item_id)
            .bind(&item.name)
            .bind(item.unit_price_amount)
            .bind(item.quantity)
            .bind(item.line_total_amount)
            .execute(&mut **transaction)
            .await
            .map_err(|error| {
                ApplicationError::unexpected_with_source(
                    "failed to insert commercial order projection item",
                    error,
                )
            })?;
        }

        Ok(())
    }

    async fn update_status(
        &self,
        tx: &mut dyn TransactionContext,
        ordering_order_id: &str,
        status: &str,
        updated_at: Timestamp,
    ) -> Result<(), ApplicationError> {
        let transaction = Self::transaction(tx)?;
        let ordering_order_id = Self::parse_uuid(ordering_order_id, "ordering order id")?;

        let result = sqlx::query(
            r#"
            UPDATE fulfillment.ordering_order_projections
            SET status = $2, updated_at = $3
            WHERE ordering_order_id = $1
            "#,
        )
        .bind(ordering_order_id)
        .bind(status)
        .bind(updated_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source(
                "failed to update commercial order projection status",
                error,
            )
        })?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::not_found(
                "commercial order projection was not found",
            ));
        }

        Ok(())
    }
}
