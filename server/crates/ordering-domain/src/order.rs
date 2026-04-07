use crate::{CatalogItemId, CustomerId, DomainError, OrderId, OrderItem, OrderStatus, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceOrderItemInput {
    pub catalog_item_id: CatalogItemId,
    pub name: String,
    pub unit_price_amount: i64,
    pub quantity: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    id: OrderId,
    customer_id: CustomerId,
    store_id: StoreId,
    status: OrderStatus,
    items: Vec<OrderItem>,
    subtotal_amount: i64,
    total_amount: i64,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Order {
    pub fn place(
        id: OrderId,
        customer_id: CustomerId,
        store_id: StoreId,
        items: Vec<PlaceOrderItemInput>,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        let items = build_items(items)?;
        let subtotal_amount = items.iter().map(OrderItem::line_total_amount).sum::<i64>();

        Ok(Self {
            id,
            customer_id,
            store_id,
            status: OrderStatus::Placed,
            items,
            subtotal_amount,
            total_amount: subtotal_amount,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: OrderId,
        customer_id: CustomerId,
        store_id: StoreId,
        status: OrderStatus,
        items: Vec<OrderItem>,
        subtotal_amount: i64,
        total_amount: i64,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Result<Self, DomainError> {
        if items.is_empty() {
            return Err(DomainError::EmptyOrderItems);
        }

        let expected_subtotal = items.iter().map(OrderItem::line_total_amount).sum::<i64>();
        if subtotal_amount != expected_subtotal {
            return Err(DomainError::InvalidSubtotalAmount);
        }
        if total_amount != subtotal_amount {
            return Err(DomainError::InvalidTotalAmount);
        }

        Ok(Self {
            id,
            customer_id,
            store_id,
            status,
            items,
            subtotal_amount,
            total_amount,
            created_at,
            updated_at,
        })
    }

    pub fn cancel_by_customer(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.cancel_by_customer()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn id(&self) -> &OrderId {
        &self.id
    }

    pub fn customer_id(&self) -> &CustomerId {
        &self.customer_id
    }

    pub fn store_id(&self) -> &StoreId {
        &self.store_id
    }

    pub fn status(&self) -> OrderStatus {
        self.status
    }

    pub fn items(&self) -> &[OrderItem] {
        &self.items
    }

    pub fn subtotal_amount(&self) -> i64 {
        self.subtotal_amount
    }

    pub fn total_amount(&self) -> i64 {
        self.total_amount
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }
}

fn build_items(inputs: Vec<PlaceOrderItemInput>) -> Result<Vec<OrderItem>, DomainError> {
    if inputs.is_empty() {
        return Err(DomainError::EmptyOrderItems);
    }

    inputs
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            OrderItem::create(
                (index + 1) as i32,
                item.catalog_item_id,
                item.name,
                item.unit_price_amount,
                item.quantity,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn make_order(now: Timestamp) -> Order {
        Order::place(
            OrderId::new("order-1"),
            CustomerId::new("customer-1"),
            StoreId::new("store-1"),
            vec![PlaceOrderItemInput {
                catalog_item_id: CatalogItemId::new("item-1"),
                name: "Fried Rice".to_string(),
                unit_price_amount: 3200,
                quantity: 2,
            }],
            now,
        )
        .unwrap()
    }

    #[test]
    fn order_requires_at_least_one_item() {
        let error = Order::place(
            OrderId::new("order-1"),
            CustomerId::new("customer-1"),
            StoreId::new("store-1"),
            Vec::new(),
            datetime!(2026-03-15 09:00 UTC),
        )
        .unwrap_err();

        assert_eq!(error, DomainError::EmptyOrderItems);
    }

    #[test]
    fn order_snapshot_is_not_affected_by_source_changes() {
        let mut name = "Fried Rice".to_string();
        let order = Order::place(
            OrderId::new("order-1"),
            CustomerId::new("customer-1"),
            StoreId::new("store-1"),
            vec![PlaceOrderItemInput {
                catalog_item_id: CatalogItemId::new("item-1"),
                name: name.clone(),
                unit_price_amount: 3200,
                quantity: 1,
            }],
            datetime!(2026-03-15 09:00 UTC),
        )
        .unwrap();

        name.clear();

        assert_eq!(order.items()[0].name(), "Fried Rice");
        assert_eq!(order.total_amount(), 3200);
    }

    #[test]
    fn customer_can_cancel_placed_order() {
        let mut order = make_order(datetime!(2026-03-15 09:00 UTC));
        order
            .cancel_by_customer(datetime!(2026-03-15 09:01 UTC))
            .unwrap();

        assert_eq!(order.status(), OrderStatus::CancelledByCustomer);
    }

    #[test]
    fn customer_cannot_cancel_twice() {
        let mut order = make_order(datetime!(2026-03-15 09:00 UTC));
        order
            .cancel_by_customer(datetime!(2026-03-15 09:01 UTC))
            .unwrap();

        let error = order
            .cancel_by_customer(datetime!(2026-03-15 09:02 UTC))
            .unwrap_err();

        assert_eq!(
            error,
            DomainError::InvalidTransition {
                event: "cancel_by_customer".to_string(),
                status: "cancelled_by_customer".to_string(),
            }
        );
    }

    #[test]
    fn rehydrate_validates_totals() {
        let item =
            OrderItem::create(1, CatalogItemId::new("item-1"), "Fried Rice", 3200, 2).unwrap();

        let error = Order::rehydrate(
            OrderId::new("order-1"),
            CustomerId::new("customer-1"),
            StoreId::new("store-1"),
            OrderStatus::Placed,
            vec![item],
            1000,
            1000,
            datetime!(2026-03-15 09:00 UTC),
            datetime!(2026-03-15 09:01 UTC),
        )
        .unwrap_err();

        assert_eq!(error, DomainError::InvalidSubtotalAmount);
    }
}
