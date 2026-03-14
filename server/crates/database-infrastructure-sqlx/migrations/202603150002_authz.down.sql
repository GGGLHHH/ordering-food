DROP INDEX IF EXISTS idx_authz_store_memberships_store_role;
DROP INDEX IF EXISTS idx_authz_store_memberships_user_store;
DROP INDEX IF EXISTS idx_authz_user_global_roles_user_id;
DROP TABLE IF EXISTS authz.store_memberships;
DROP TABLE IF EXISTS authz.user_global_roles;
DROP TYPE IF EXISTS authz.store_role;
DROP TYPE IF EXISTS authz.global_role;
DROP SCHEMA IF EXISTS authz;
