CREATE SCHEMA IF NOT EXISTS organization;

CREATE TABLE organization.brands (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT organization_brands_slug_unique UNIQUE (slug)
);

CREATE TABLE organization.stores (
    id UUID PRIMARY KEY,
    brand_id UUID NOT NULL REFERENCES organization.brands(id),
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    currency_code CHAR(3) NOT NULL,
    timezone TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT organization_stores_brand_slug_unique UNIQUE (brand_id, slug)
);

INSERT INTO organization.brands (
    id,
    slug,
    name,
    status,
    created_at,
    updated_at,
    deleted_at
)
VALUES (
    '00000000-0000-4000-8000-000000000001',
    'ordering-food',
    'Ordering Food',
    'active',
    NOW(),
    NOW(),
    NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO organization.stores (
    id,
    brand_id,
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
    '00000000-0000-4000-8000-000000000001'::UUID,
    slug,
    name,
    currency_code,
    timezone,
    status,
    created_at,
    updated_at,
    deleted_at
FROM menu.stores
ON CONFLICT (id) DO NOTHING;

DROP TABLE menu.stores;

CREATE VIEW menu.stores AS
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
