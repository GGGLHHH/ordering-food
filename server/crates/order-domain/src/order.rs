use crate::{CustomerId, DomainError, MenuItemId, OrderId, OrderItem, OrderStatus, StoreId};
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceOrderItemInput {
    pub menu_item_id: MenuItemId,
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
            status: OrderStatus::PendingAcceptance,
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

    pub fn accept(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.accept()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn start_preparing(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.start_preparing()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn mark_ready(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.mark_ready()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn complete(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.complete()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn cancel_by_customer(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.cancel_by_customer()?;
        self.updated_at = now;
        Ok(())
    }

    pub fn reject_by_store(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.status = self.status.reject_by_store()?;
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
                item.menu_item_id,
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
                menu_item_id: MenuItemId::new("item-1"),
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
                menu_item_id: MenuItemId::new("item-1"),
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
    fn order_allows_happy_path_transitions() {
        let now = datetime!(2026-03-15 09:00 UTC);
        let mut order = make_order(now);

        order.accept(datetime!(2026-03-15 09:01 UTC)).unwrap();
        order
            .start_preparing(datetime!(2026-03-15 09:02 UTC))
            .unwrap();
        order.mark_ready(datetime!(2026-03-15 09:03 UTC)).unwrap();
        order.complete(datetime!(2026-03-15 09:04 UTC)).unwrap();

        assert_eq!(order.status(), OrderStatus::Completed);
    }

    #[test]
    fn customer_can_cancel_before_preparing() {
        let mut order = make_order(datetime!(2026-03-15 09:00 UTC));
        order.accept(datetime!(2026-03-15 09:01 UTC)).unwrap();
        order
            .cancel_by_customer(datetime!(2026-03-15 09:02 UTC))
            .unwrap();

        assert_eq!(order.status(), OrderStatus::CancelledByCustomer);
    }

    #[test]
    fn customer_cannot_cancel_after_preparing() {
        let mut order = make_order(datetime!(2026-03-15 09:00 UTC));
        order.accept(datetime!(2026-03-15 09:01 UTC)).unwrap();
        order
            .start_preparing(datetime!(2026-03-15 09:02 UTC))
            .unwrap();

        let error = order
            .cancel_by_customer(datetime!(2026-03-15 09:03 UTC))
            .unwrap_err();

        assert_eq!(
            error,
            DomainError::InvalidTransition {
                event: "cancel_by_customer".to_string(),
                status: "preparing".to_string(),
            }
        );
    }

    #[test]
    fn rejected_order_cannot_progress() {
        let mut order = make_order(datetime!(2026-03-15 09:00 UTC));
        order
            .reject_by_store(datetime!(2026-03-15 09:01 UTC))
            .unwrap();

        let error = order.accept(datetime!(2026-03-15 09:02 UTC)).unwrap_err();

        assert_eq!(
            error,
            DomainError::InvalidTransition {
                event: "accept".to_string(),
                status: "rejected_by_store".to_string(),
            }
        );
    }

    #[test]
    fn order_can_only_complete_when_ready_for_pickup() {
        let mut order = make_order(datetime!(2026-03-15 09:00 UTC));
        order.accept(datetime!(2026-03-15 09:01 UTC)).unwrap();

        let error = order.complete(datetime!(2026-03-15 09:02 UTC)).unwrap_err();

        assert_eq!(
            error,
            DomainError::InvalidTransition {
                event: "complete".to_string(),
                status: "accepted".to_string(),
            }
        );
    }
}
