DROP INDEX IF EXISTS access.idx_access_store_memberships_store_role;
DROP INDEX IF EXISTS access.idx_access_store_memberships_subject_store;
DROP INDEX IF EXISTS access.idx_access_subject_global_roles_subject_id;
DROP TABLE IF EXISTS access.store_memberships;
DROP TABLE IF EXISTS access.subject_global_roles;
DROP TYPE IF EXISTS access.store_role;
DROP TYPE IF EXISTS access.global_role;
DROP SCHEMA IF EXISTS access;
