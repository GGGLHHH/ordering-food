CREATE EXTENSION IF NOT EXISTS pgcrypto;

UPDATE fulfillment.workflow_orders
SET id = gen_random_uuid()
WHERE id = ordering_order_id;
