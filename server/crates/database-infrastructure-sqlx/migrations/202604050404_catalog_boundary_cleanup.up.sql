ALTER TABLE catalog.brand_catalogs
    DROP CONSTRAINT IF EXISTS brand_catalogs_brand_id_fkey;

ALTER TABLE catalog.store_catalogs
    DROP CONSTRAINT IF EXISTS store_catalogs_brand_id_fkey;

ALTER TABLE catalog.store_catalogs
    DROP CONSTRAINT IF EXISTS store_catalogs_store_id_fkey;
