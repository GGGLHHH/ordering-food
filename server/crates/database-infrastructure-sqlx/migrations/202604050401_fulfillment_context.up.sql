CREATE SCHEMA IF NOT EXISTS fulfillment;

CREATE TYPE fulfillment.workflow_status AS ENUM (
    'pending_acceptance',
    'accepted',
    'preparing',
    'ready_for_pickup',
    'completed',
    'cancelled_by_customer',
    'rejected_by_store'
);

CREATE TABLE fulfillment.workflow_orders (
    id UUID PRIMARY KEY,
    ordering_order_id UUID NOT NULL UNIQUE,
    store_id UUID NOT NULL,
    status fulfillment.workflow_status NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_fulfillment_workflow_orders_store_status_created_at
    ON fulfillment.workflow_orders (store_id, status, created_at DESC, id DESC);

INSERT INTO fulfillment.workflow_orders (
    id,
    ordering_order_id,
    store_id,
    status,
    created_at,
    updated_at
)
SELECT
    o.id,
    o.id,
    o.store_id,
    (o.status::text)::fulfillment.workflow_status,
    o.created_at,
    o.updated_at
FROM ordering.orders o;

COMMENT ON TABLE fulfillment.workflow_orders IS
    'Store-side workflow truth introduced in Phase 2D. Phase 3 will replace temporary sync bootstrap with events.';
