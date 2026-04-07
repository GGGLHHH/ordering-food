ALTER TABLE ordering.order_items
    RENAME COLUMN menu_item_id TO catalog_item_id;

CREATE TYPE ordering.commercial_order_status AS ENUM (
    'placed',
    'cancelled_by_customer'
);

DROP INDEX IF EXISTS idx_ordering_orders_store_status_created_at;

ALTER TABLE ordering.orders
    ALTER COLUMN status TYPE ordering.commercial_order_status
    USING (
        CASE
            WHEN status::text = 'cancelled_by_customer' THEN 'cancelled_by_customer'
            ELSE 'placed'
        END
    )::ordering.commercial_order_status;

DROP TYPE ordering.order_status;
ALTER TYPE ordering.commercial_order_status RENAME TO order_status;

CREATE INDEX idx_ordering_orders_store_created_at
    ON ordering.orders (store_id, created_at DESC, id DESC);

COMMENT ON TABLE ordering.orders IS
    'Commercial order truth only after Phase 2D. Store workflow truth moved to fulfillment.workflow_orders.';

COMMENT ON COLUMN ordering.orders.status IS
    'Commercial state only after Phase 2D. Phase 3 removes temporary sync seams via event collaboration.';
