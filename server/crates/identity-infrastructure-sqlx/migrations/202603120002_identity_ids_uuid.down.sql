ALTER TABLE identity.user_credentials
    DROP CONSTRAINT IF EXISTS user_credentials_user_id_fkey;

ALTER TABLE identity.user_identities
    DROP CONSTRAINT IF EXISTS user_identities_user_id_fkey;

ALTER TABLE identity.user_profiles
    DROP CONSTRAINT IF EXISTS user_profiles_user_id_fkey;

ALTER TABLE identity.user_credentials
    ALTER COLUMN user_id TYPE TEXT USING user_id::text;

ALTER TABLE identity.user_identities
    ALTER COLUMN user_id TYPE TEXT USING user_id::text;

ALTER TABLE identity.user_profiles
    ALTER COLUMN user_id TYPE TEXT USING user_id::text;

ALTER TABLE identity.users
    ALTER COLUMN id TYPE TEXT USING id::text;

ALTER TABLE identity.user_profiles
    ADD CONSTRAINT user_profiles_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES identity.users(id) ON DELETE CASCADE;

ALTER TABLE identity.user_identities
    ADD CONSTRAINT user_identities_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES identity.users(id) ON DELETE CASCADE;

ALTER TABLE identity.user_credentials
    ADD CONSTRAINT user_credentials_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES identity.users(id) ON DELETE CASCADE;
