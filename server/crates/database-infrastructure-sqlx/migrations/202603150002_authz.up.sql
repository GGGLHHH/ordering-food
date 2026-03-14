CREATE SCHEMA IF NOT EXISTS authz;

CREATE TYPE authz.global_role AS ENUM (
    'platform_admin'
);

CREATE TYPE authz.store_role AS ENUM (
    'store_owner',
    'store_staff'
);

CREATE TABLE authz.user_global_roles (
    user_id UUID NOT NULL,
    role authz.global_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, role)
);

CREATE TABLE authz.store_memberships (
    user_id UUID NOT NULL,
    store_id UUID NOT NULL,
    role authz.store_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, store_id, role)
);

CREATE INDEX idx_authz_user_global_roles_user_id
    ON authz.user_global_roles (user_id);

CREATE INDEX idx_authz_store_memberships_user_store
    ON authz.store_memberships (user_id, store_id);

CREATE INDEX idx_authz_store_memberships_store_role
    ON authz.store_memberships (store_id, role);
