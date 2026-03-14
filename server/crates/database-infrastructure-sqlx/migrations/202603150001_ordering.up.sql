CREATE SCHEMA IF NOT EXISTS ordering;

CREATE TYPE ordering.order_status AS ENUM (
    'pending_acceptance',
    'accepted',
    'preparing',
    'ready_for_pickup',
    'completed',
    'cancelled_by_customer',
    'rejected_by_store'
);

CREATE TABLE ordering.orders (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL,
    store_id UUID NOT NULL,
    status ordering.order_status NOT NULL,
    subtotal_amount BIGINT NOT NULL CHECK (subtotal_amount >= 0),
    total_amount BIGINT NOT NULL CHECK (total_amount >= 0),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE ordering.order_items (
    order_id UUID NOT NULL REFERENCES ordering.orders(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL CHECK (line_number > 0),
    menu_item_id UUID NOT NULL,
    name TEXT NOT NULL,
    unit_price_amount BIGINT NOT NULL CHECK (unit_price_amount >= 0),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    line_total_amount BIGINT NOT NULL CHECK (line_total_amount >= 0),
    PRIMARY KEY (order_id, line_number)
);

CREATE INDEX idx_ordering_orders_customer_created_at
    ON ordering.orders (customer_id, created_at DESC, id DESC);

CREATE INDEX idx_ordering_orders_store_status_created_at
    ON ordering.orders (store_id, status, created_at DESC, id DESC);

