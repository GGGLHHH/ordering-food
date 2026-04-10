use async_trait::async_trait;
use ordering_food_ordering_application::{
    ApplicationError, OrderItemReadModel, OrderListItemReadModel, OrderQueryService,
    OrderReadModel, OrderReadRepository,
};
use ordering_food_shared_kernel::Timestamp;
use std::{collections::HashMap, sync::Arc};
use time::macros::datetime;

#[tokio::test]
async fn customer_visible_query_returns_order_for_owner() {
    let service = OrderQueryService::new(Arc::new(FakeOrderReadRepository::with_order(
        order_read_model("order-1", "customer-1"),
    )));

    let order = service
        .get_by_id_for_customer("order-1", "customer-1")
        .await
        .unwrap();

    assert_eq!(order.order_id, "order-1");
    assert_eq!(order.customer_id, "customer-1");
}

#[tokio::test]
async fn customer_visible_query_hides_other_customers_order() {
    let service = OrderQueryService::new(Arc::new(FakeOrderReadRepository::with_order(
        order_read_model("order-1", "customer-1"),
    )));

    let error = service
        .get_by_id_for_customer("order-1", "other-customer")
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
}

#[tokio::test]
async fn customer_visible_query_returns_not_found_when_order_missing() {
    let service = OrderQueryService::new(Arc::new(FakeOrderReadRepository::default()));

    let error = service
        .get_by_id_for_customer("missing-order", "customer-1")
        .await
        .unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
}

#[derive(Default)]
struct FakeOrderReadRepository {
    orders: HashMap<String, OrderReadModel>,
}

impl FakeOrderReadRepository {
    fn with_order(order: OrderReadModel) -> Self {
        let mut orders = HashMap::new();
        orders.insert(order.order_id.clone(), order);
        Self { orders }
    }
}

#[async_trait]
impl OrderReadRepository for FakeOrderReadRepository {
    async fn get_by_id(&self, order_id: &str) -> Result<Option<OrderReadModel>, ApplicationError> {
        Ok(self.orders.get(order_id).cloned())
    }

    async fn list_by_customer(
        &self,
        customer_id: &str,
    ) -> Result<Vec<OrderListItemReadModel>, ApplicationError> {
        Ok(self
            .orders
            .values()
            .filter(|order| order.customer_id == customer_id)
            .map(|order| OrderListItemReadModel {
                order_id: order.order_id.clone(),
                customer_id: order.customer_id.clone(),
                store_id: order.store_id.clone(),
                status: order.status.clone(),
                subtotal_amount: order.subtotal_amount,
                total_amount: order.total_amount,
                created_at: order.created_at,
                updated_at: order.updated_at,
                item_count: order.items.len(),
            })
            .collect())
    }
}

fn order_read_model(order_id: &str, customer_id: &str) -> OrderReadModel {
    OrderReadModel {
        order_id: order_id.to_string(),
        customer_id: customer_id.to_string(),
        store_id: "store-1".to_string(),
        status: "placed".to_string(),
        subtotal_amount: 3200,
        total_amount: 3200,
        created_at: fixed_timestamp(),
        updated_at: fixed_timestamp(),
        items: vec![OrderItemReadModel {
            line_number: 1,
            catalog_item_id: "item-1".to_string(),
            name: "Fried Rice".to_string(),
            unit_price_amount: 3200,
            quantity: 1,
            line_total_amount: 3200,
        }],
    }
}

fn fixed_timestamp() -> Timestamp {
    datetime!(2026-04-09 12:00 UTC)
}
