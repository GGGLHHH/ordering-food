ALTER TABLE identity.user_credentials
    DROP CONSTRAINT IF EXISTS user_credentials_user_id_fkey;

ALTER TABLE identity.user_identities
    DROP CONSTRAINT IF EXISTS user_identities_user_id_fkey;

ALTER TABLE identity.user_profiles
    DROP CONSTRAINT IF EXISTS user_profiles_user_id_fkey;
