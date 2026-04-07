CREATE TABLE fulfillment.ordering_order_projections (
    ordering_order_id UUID PRIMARY KEY,
    customer_id UUID NOT NULL,
    store_id UUID NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('placed', 'cancelled_by_customer')),
    subtotal_amount BIGINT NOT NULL CHECK (subtotal_amount >= 0),
    total_amount BIGINT NOT NULL CHECK (total_amount >= 0),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_fulfillment_ordering_order_projections_store_created_at
    ON fulfillment.ordering_order_projections (store_id, created_at DESC, ordering_order_id DESC);

CREATE TABLE fulfillment.ordering_order_projection_items (
    ordering_order_id UUID NOT NULL
        REFERENCES fulfillment.ordering_order_projections(ordering_order_id)
        ON DELETE CASCADE,
    line_number INTEGER NOT NULL CHECK (line_number > 0),
    catalog_item_id UUID NOT NULL,
    name TEXT NOT NULL,
    unit_price_amount BIGINT NOT NULL CHECK (unit_price_amount >= 0),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    line_total_amount BIGINT NOT NULL CHECK (line_total_amount >= 0),
    PRIMARY KEY (ordering_order_id, line_number)
);

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
SELECT
    orders.id,
    orders.customer_id,
    orders.store_id,
    orders.status::text,
    orders.subtotal_amount,
    orders.total_amount,
    orders.created_at,
    orders.updated_at
FROM ordering.orders AS orders
ON CONFLICT (ordering_order_id) DO NOTHING;

INSERT INTO fulfillment.ordering_order_projection_items (
    ordering_order_id,
    line_number,
    catalog_item_id,
    name,
    unit_price_amount,
    quantity,
    line_total_amount
)
SELECT
    items.order_id,
    items.line_number,
    items.catalog_item_id,
    items.name,
    items.unit_price_amount,
    items.quantity,
    items.line_total_amount
FROM ordering.order_items AS items
ON CONFLICT (ordering_order_id, line_number) DO NOTHING;
