CREATE SCHEMA IF NOT EXISTS catalog;

CREATE TABLE catalog.brand_catalogs (
    id UUID PRIMARY KEY,
    brand_id UUID NOT NULL,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT catalog_brand_catalogs_brand_id_unique UNIQUE (brand_id),
    CONSTRAINT catalog_brand_catalogs_slug_unique UNIQUE (slug)
);

CREATE INDEX idx_catalog_brand_catalogs_brand_id
    ON catalog.brand_catalogs (brand_id);

CREATE TABLE catalog.store_catalogs (
    id UUID PRIMARY KEY,
    brand_id UUID NOT NULL,
    store_id UUID NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('sellable', 'unsellable')),
    display_rule TEXT NOT NULL CHECK (display_rule IN ('listed', 'hidden')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT catalog_store_catalogs_store_id_unique UNIQUE (store_id)
);

CREATE INDEX idx_catalog_store_catalogs_store_id
    ON catalog.store_catalogs (store_id);

CREATE TABLE catalog.categories (
    id UUID PRIMARY KEY,
    brand_catalog_id UUID NOT NULL REFERENCES catalog.brand_catalogs(id) ON DELETE CASCADE,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT catalog_categories_brand_catalog_slug_unique UNIQUE (brand_catalog_id, slug)
);

CREATE INDEX idx_catalog_categories_brand_catalog_sort
    ON catalog.categories (brand_catalog_id, sort_order, id);

CREATE TABLE catalog.items (
    id UUID PRIMARY KEY,
    brand_catalog_id UUID NOT NULL REFERENCES catalog.brand_catalogs(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES catalog.categories(id) ON DELETE CASCADE,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NULL,
    image_url TEXT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT catalog_items_brand_catalog_slug_unique UNIQUE (brand_catalog_id, slug)
);

CREATE INDEX idx_catalog_items_brand_catalog_sort
    ON catalog.items (brand_catalog_id, sort_order, id);

CREATE INDEX idx_catalog_items_category_sort
    ON catalog.items (category_id, sort_order, id);

CREATE TABLE catalog.store_item_listings (
    store_catalog_id UUID NOT NULL REFERENCES catalog.store_catalogs(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES catalog.items(id) ON DELETE CASCADE,
    price_amount BIGINT NOT NULL CHECK (price_amount >= 0),
    status TEXT NOT NULL CHECK (status IN ('sellable', 'unsellable')),
    display_rule TEXT NOT NULL CHECK (display_rule IN ('listed', 'hidden')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (store_catalog_id, item_id)
);

CREATE INDEX idx_catalog_store_item_listings_store_catalog
    ON catalog.store_item_listings (store_catalog_id, status, display_rule, item_id);

INSERT INTO catalog.brand_catalogs (
    id,
    brand_id,
    slug,
    name,
    created_at,
    updated_at
)
SELECT
    brands.id,
    brands.id,
    brands.slug,
    brands.name,
    brands.created_at,
    brands.updated_at
FROM organization.brands AS brands
ON CONFLICT (id) DO NOTHING;

INSERT INTO catalog.store_catalogs (
    id,
    brand_id,
    store_id,
    status,
    display_rule,
    created_at,
    updated_at
)
SELECT
    stores.id,
    stores.brand_id,
    stores.id,
    CASE
        WHEN stores.status = 'active' THEN 'sellable'
        ELSE 'unsellable'
    END AS status,
    CASE
        WHEN stores.status = 'active' THEN 'listed'
        ELSE 'hidden'
    END AS display_rule,
    stores.created_at,
    stores.updated_at
FROM organization.stores AS stores
ON CONFLICT (id) DO NOTHING;

WITH projected_categories AS (
    SELECT DISTINCT ON (brand_catalogs.id, categories.slug)
        categories.id,
        brand_catalogs.id AS brand_catalog_id,
        categories.slug,
        categories.name,
        categories.description,
        categories.sort_order,
        categories.created_at,
        categories.updated_at
    FROM menu.categories AS categories
    INNER JOIN organization.stores AS stores
        ON stores.id = categories.store_id
    INNER JOIN catalog.brand_catalogs AS brand_catalogs
        ON brand_catalogs.brand_id = stores.brand_id
    WHERE categories.deleted_at IS NULL
    ORDER BY brand_catalogs.id, categories.slug, categories.created_at ASC, categories.id ASC
)
INSERT INTO catalog.categories (
    id,
    brand_catalog_id,
    slug,
    name,
    description,
    sort_order,
    created_at,
    updated_at
)
SELECT
    projected_categories.id,
    projected_categories.brand_catalog_id,
    projected_categories.slug,
    projected_categories.name,
    projected_categories.description,
    projected_categories.sort_order,
    projected_categories.created_at,
    projected_categories.updated_at
FROM projected_categories
ON CONFLICT (id) DO NOTHING;

WITH projected_items AS (
    SELECT DISTINCT ON (brand_catalogs.id, items.slug)
        items.id,
        brand_catalogs.id AS brand_catalog_id,
        catalog_categories.id AS category_id,
        items.slug,
        items.name,
        items.description,
        items.image_url,
        items.sort_order,
        items.created_at,
        items.updated_at
    FROM menu.items AS items
    INNER JOIN menu.categories AS categories
        ON categories.id = items.category_id
    INNER JOIN organization.stores AS stores
        ON stores.id = items.store_id
    INNER JOIN catalog.brand_catalogs AS brand_catalogs
        ON brand_catalogs.brand_id = stores.brand_id
    INNER JOIN catalog.categories AS catalog_categories
        ON catalog_categories.brand_catalog_id = brand_catalogs.id
       AND catalog_categories.slug = categories.slug
    WHERE items.deleted_at IS NULL
      AND categories.deleted_at IS NULL
    ORDER BY brand_catalogs.id, items.slug, items.created_at ASC, items.id ASC
)
INSERT INTO catalog.items (
    id,
    brand_catalog_id,
    category_id,
    slug,
    name,
    description,
    image_url,
    sort_order,
    created_at,
    updated_at
)
SELECT
    projected_items.id,
    projected_items.brand_catalog_id,
    projected_items.category_id,
    projected_items.slug,
    projected_items.name,
    projected_items.description,
    projected_items.image_url,
    projected_items.sort_order,
    projected_items.created_at,
    projected_items.updated_at
FROM projected_items
ON CONFLICT (id) DO NOTHING;

INSERT INTO catalog.store_item_listings (
    store_catalog_id,
    item_id,
    price_amount,
    status,
    display_rule,
    created_at,
    updated_at
)
SELECT
    store_catalogs.id,
    catalog_items.id,
    items.price_amount,
    CASE
        WHEN items.status = 'active' THEN 'sellable'
        ELSE 'unsellable'
    END AS status,
    CASE
        WHEN items.status = 'active' THEN 'listed'
        ELSE 'hidden'
    END AS display_rule,
    items.created_at,
    items.updated_at
FROM menu.items AS items
INNER JOIN catalog.store_catalogs AS store_catalogs
    ON store_catalogs.store_id = items.store_id
INNER JOIN catalog.brand_catalogs AS brand_catalogs
    ON brand_catalogs.brand_id = store_catalogs.brand_id
INNER JOIN catalog.items AS catalog_items
    ON catalog_items.brand_catalog_id = brand_catalogs.id
   AND catalog_items.slug = items.slug
WHERE items.deleted_at IS NULL
ON CONFLICT (store_catalog_id, item_id) DO NOTHING;
