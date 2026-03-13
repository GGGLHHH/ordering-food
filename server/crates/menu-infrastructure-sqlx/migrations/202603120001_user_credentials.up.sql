CREATE TABLE identity.user_credentials (
    user_id       TEXT PRIMARY KEY REFERENCES identity.users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL,
    updated_at    TIMESTAMPTZ NOT NULL
);
