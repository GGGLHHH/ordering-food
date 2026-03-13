CREATE SCHEMA IF NOT EXISTS identity;

CREATE TABLE identity.users (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL CHECK (status IN ('active', 'disabled')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL
);

CREATE TABLE identity.user_profiles (
    user_id TEXT PRIMARY KEY REFERENCES identity.users(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    given_name TEXT NULL,
    family_name TEXT NULL,
    avatar_url TEXT NULL
);

CREATE TABLE identity.user_identities (
    user_id TEXT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    identity_type TEXT NOT NULL,
    identifier_normalized TEXT NOT NULL,
    bound_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, identity_type, identifier_normalized),
    CONSTRAINT user_identities_identifier_unique UNIQUE (identity_type, identifier_normalized)
);

CREATE INDEX idx_user_identities_user_id ON identity.user_identities (user_id);
