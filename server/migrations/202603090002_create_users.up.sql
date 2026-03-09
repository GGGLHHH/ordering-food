CREATE TABLE users (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    phone      VARCHAR(20)  NOT NULL UNIQUE,
    nickname   VARCHAR(50)  NOT NULL DEFAULT '',
    avatar_url TEXT         NOT NULL DEFAULT '',
    role       VARCHAR(20)  NOT NULL DEFAULT 'customer',
    status     VARCHAR(20)  NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
