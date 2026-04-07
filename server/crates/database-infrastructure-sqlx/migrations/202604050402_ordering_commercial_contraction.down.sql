-- Best-effort rollback: Phase 2D collapses workflow progression into commercial state only.
-- Recreating the old ordering workflow enum cannot recover accepted/preparing/ready/completed history.

DROP INDEX IF EXISTS idx_ordering_orders_store_created_at;

CREATE TYPE ordering.order_status_legacy AS ENUM (
    'pending_acceptance',
    'accepted',
    'preparing',
    'ready_for_pickup',
    'completed',
    'cancelled_by_customer',
    'rejected_by_store'
);

ALTER TABLE ordering.orders
    ALTER COLUMN status TYPE ordering.order_status_legacy
    USING (
        CASE
            WHEN status::text = 'cancelled_by_customer' THEN 'cancelled_by_customer'
            ELSE 'pending_acceptance'
        END
    )::ordering.order_status_legacy;

DROP TYPE ordering.order_status;
ALTER TYPE ordering.order_status_legacy RENAME TO order_status;

ALTER TABLE ordering.order_items
    RENAME COLUMN catalog_item_id TO menu_item_id;

CREATE INDEX idx_ordering_orders_store_status_created_at
    ON ordering.orders (store_id, status, created_at DESC, id DESC);
