CREATE SCHEMA IF NOT EXISTS menu;

CREATE TABLE menu.stores (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    currency_code CHAR(3) NOT NULL,
    timezone TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT stores_slug_unique UNIQUE (slug)
);

CREATE TABLE menu.categories (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT categories_store_slug_unique UNIQUE (store_id, slug)
);

CREATE TABLE menu.items (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL,
    category_id UUID NOT NULL,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NULL,
    image_url TEXT NULL,
    price_amount BIGINT NOT NULL CHECK (price_amount >= 0),
    sort_order INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT items_store_slug_unique UNIQUE (store_id, slug)
);

CREATE INDEX idx_menu_categories_store_status_sort
    ON menu.categories (store_id, status, sort_order, id);

CREATE INDEX idx_menu_items_store_status_sort
    ON menu.items (store_id, status, sort_order, id);

CREATE INDEX idx_menu_items_category_status_sort
    ON menu.items (category_id, status, sort_order, id);

CREATE INDEX idx_menu_items_store_category_status_sort
    ON menu.items (store_id, category_id, status, sort_order, id);
