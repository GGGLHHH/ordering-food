CREATE SCHEMA IF NOT EXISTS menu;

-- Best-effort rollback: store-scoped menu IDs are deterministically rebuilt from
-- store_id + catalog IDs. Legacy menu category/item IDs are not preserved.
-- Keep stores as a view so the older organization rollback can still drop and recreate it.
CREATE OR REPLACE VIEW menu.stores AS
SELECT
    id,
    slug,
    name,
    currency_code,
    timezone,
    status,
    created_at,
    updated_at,
    deleted_at
FROM organization.stores;

-- Categories and items stay materialized because the legacy menu runtime writes to them.
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
    category_id UUID NOT NULL REFERENCES menu.categories(id) ON DELETE CASCADE,
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

WITH reconstructed_categories AS (
    SELECT
        stores.id AS store_id,
        catalog_categories.id AS catalog_category_id,
        md5(stores.id::text || ':' || catalog_categories.id::text) AS category_hash,
        catalog_categories.slug,
        catalog_categories.name,
        catalog_categories.description,
        catalog_categories.sort_order,
        CASE
            WHEN stores.status = 'active' THEN 'active'
            ELSE 'inactive'
        END AS status,
        catalog_categories.created_at,
        catalog_categories.updated_at
    FROM catalog.categories AS catalog_categories
    INNER JOIN catalog.brand_catalogs AS brand_catalogs
        ON brand_catalogs.id = catalog_categories.brand_catalog_id
    INNER JOIN organization.stores AS stores
        ON stores.brand_id = brand_catalogs.brand_id
)
INSERT INTO menu.categories (
    id,
    store_id,
    slug,
    name,
    description,
    sort_order,
    status,
    created_at,
    updated_at,
    deleted_at
)
SELECT
    (
        substr(reconstructed_categories.category_hash, 1, 8) || '-' ||
        substr(reconstructed_categories.category_hash, 9, 4) || '-' ||
        substr(reconstructed_categories.category_hash, 13, 4) || '-' ||
        substr(reconstructed_categories.category_hash, 17, 4) || '-' ||
        substr(reconstructed_categories.category_hash, 21, 12)
    )::UUID AS id,
    reconstructed_categories.store_id,
    reconstructed_categories.slug,
    reconstructed_categories.name,
    reconstructed_categories.description,
    reconstructed_categories.sort_order,
    reconstructed_categories.status,
    reconstructed_categories.created_at,
    reconstructed_categories.updated_at,
    NULL
FROM reconstructed_categories;

WITH reconstructed_categories AS (
    SELECT
        reconstructed.store_id,
        reconstructed.catalog_category_id,
        (
            substr(reconstructed.category_hash, 1, 8) || '-' ||
            substr(reconstructed.category_hash, 9, 4) || '-' ||
            substr(reconstructed.category_hash, 13, 4) || '-' ||
            substr(reconstructed.category_hash, 17, 4) || '-' ||
            substr(reconstructed.category_hash, 21, 12)
        )::UUID AS menu_category_id
    FROM (
        SELECT
            stores.id AS store_id,
            catalog_categories.id AS catalog_category_id,
            md5(stores.id::text || ':' || catalog_categories.id::text) AS category_hash
        FROM catalog.categories AS catalog_categories
        INNER JOIN catalog.brand_catalogs AS brand_catalogs
            ON brand_catalogs.id = catalog_categories.brand_catalog_id
        INNER JOIN organization.stores AS stores
            ON stores.brand_id = brand_catalogs.brand_id
    ) AS reconstructed
),
reconstructed_items AS (
    SELECT
        store_catalogs.store_id,
        reconstructed_categories.menu_category_id,
        md5(store_catalogs.store_id::text || ':' || catalog_items.id::text) AS item_hash,
        catalog_items.slug,
        catalog_items.name,
        catalog_items.description,
        catalog_items.image_url,
        store_item_listings.price_amount,
        catalog_items.sort_order,
        CASE
            WHEN store_catalogs.status = 'sellable'
             AND store_item_listings.status = 'sellable' THEN 'active'
            ELSE 'inactive'
        END AS status,
        catalog_items.created_at,
        GREATEST(catalog_items.updated_at, store_item_listings.updated_at) AS updated_at
    FROM catalog.store_item_listings AS store_item_listings
    INNER JOIN catalog.store_catalogs AS store_catalogs
        ON store_catalogs.id = store_item_listings.store_catalog_id
    INNER JOIN catalog.items AS catalog_items
        ON catalog_items.id = store_item_listings.item_id
    INNER JOIN reconstructed_categories
        ON reconstructed_categories.store_id = store_catalogs.store_id
       AND reconstructed_categories.catalog_category_id = catalog_items.category_id
)
INSERT INTO menu.items (
    id,
    store_id,
    category_id,
    slug,
    name,
    description,
    image_url,
    price_amount,
    sort_order,
    status,
    created_at,
    updated_at,
    deleted_at
)
SELECT
    (
        substr(reconstructed_items.item_hash, 1, 8) || '-' ||
        substr(reconstructed_items.item_hash, 9, 4) || '-' ||
        substr(reconstructed_items.item_hash, 13, 4) || '-' ||
        substr(reconstructed_items.item_hash, 17, 4) || '-' ||
        substr(reconstructed_items.item_hash, 21, 12)
    )::UUID AS id,
    reconstructed_items.store_id,
    reconstructed_items.menu_category_id,
    reconstructed_items.slug,
    reconstructed_items.name,
    reconstructed_items.description,
    reconstructed_items.image_url,
    reconstructed_items.price_amount,
    reconstructed_items.sort_order,
    reconstructed_items.status,
    reconstructed_items.created_at,
    reconstructed_items.updated_at,
    NULL
FROM reconstructed_items;
