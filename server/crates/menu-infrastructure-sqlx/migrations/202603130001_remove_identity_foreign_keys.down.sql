ALTER TABLE identity.user_profiles
    ADD CONSTRAINT user_profiles_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES identity.users(id) ON DELETE CASCADE;

ALTER TABLE identity.user_identities
    ADD CONSTRAINT user_identities_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES identity.users(id) ON DELETE CASCADE;

ALTER TABLE identity.user_credentials
    ADD CONSTRAINT user_credentials_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES identity.users(id) ON DELETE CASCADE;
