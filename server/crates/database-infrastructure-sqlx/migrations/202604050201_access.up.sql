CREATE SCHEMA IF NOT EXISTS access;

CREATE TYPE access.global_role AS ENUM (
    'platform_admin'
);

CREATE TYPE access.store_role AS ENUM (
    'store_owner',
    'store_staff'
);

CREATE TABLE access.subject_global_roles (
    subject_id UUID NOT NULL,
    role access.global_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (subject_id, role)
);

CREATE TABLE access.store_memberships (
    subject_id UUID NOT NULL,
    store_id UUID NOT NULL,
    role access.store_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (subject_id, store_id, role)
);

CREATE INDEX idx_access_subject_global_roles_subject_id
    ON access.subject_global_roles (subject_id);

CREATE INDEX idx_access_store_memberships_subject_store
    ON access.store_memberships (subject_id, store_id);

CREATE INDEX idx_access_store_memberships_store_role
    ON access.store_memberships (store_id, role);

INSERT INTO access.subject_global_roles (subject_id, role, granted_at)
SELECT
    user_id AS subject_id,
    role::text::access.global_role,
    granted_at
FROM authz.user_global_roles
ON CONFLICT (subject_id, role) DO NOTHING;

INSERT INTO access.store_memberships (subject_id, store_id, role, granted_at)
SELECT
    user_id AS subject_id,
    store_id,
    role::text::access.store_role,
    granted_at
FROM authz.store_memberships
ON CONFLICT (subject_id, store_id, role) DO NOTHING;
