ALTER TABLE catalog.brand_catalogs
    ADD CONSTRAINT brand_catalogs_brand_id_fkey
    FOREIGN KEY (brand_id) REFERENCES organization.brands(id);

ALTER TABLE catalog.store_catalogs
    ADD CONSTRAINT store_catalogs_brand_id_fkey
    FOREIGN KEY (brand_id) REFERENCES organization.brands(id);

ALTER TABLE catalog.store_catalogs
    ADD CONSTRAINT store_catalogs_store_id_fkey
    FOREIGN KEY (store_id) REFERENCES organization.stores(id);
