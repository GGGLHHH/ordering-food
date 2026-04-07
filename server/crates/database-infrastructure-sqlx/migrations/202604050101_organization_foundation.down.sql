DROP VIEW IF EXISTS menu.stores;

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

INSERT INTO menu.stores (
    id,
    slug,
    name,
    currency_code,
    timezone,
    status,
    created_at,
    updated_at,
    deleted_at
)
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

DROP TABLE IF EXISTS organization.stores;
DROP TABLE IF EXISTS organization.brands;
DROP SCHEMA IF EXISTS organization;
