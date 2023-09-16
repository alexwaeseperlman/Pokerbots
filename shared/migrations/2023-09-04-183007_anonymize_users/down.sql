ALTER TABLE user_profiles DROP CONSTRAINT user_profiles_fk_auth_id;
ALTER TABLE user_profiles DROP CONSTRAINT user_profiles_pkey;
ALTER TABLE user_profiles ADD PRIMARY KEY (email);
ALTER TABLE user_profiles DROP CONSTRAINT user_profiles_unique_id;
ALTER TABLE user_profiles DROP COLUMN id;

ALTER TABLE bots DROP CONSTRAINT bots_uploaded_by_fk_auth_id;
ALTER TABLE bots ADD COLUMN uploaded_by_email TEXT;
UPDATE bots SET uploaded_by_email = (SELECT email FROM auth WHERE bots.uploaded_by = auth.id);
ALTER TABLE bots ALTER COLUMN uploaded_by TYPE TEXT USING uploaded_by_email;
ALTER TABLE bots DROP COLUMN uploaded_by_email;

ALTER TABLE users ADD COLUMN email TEXT NOT NULL DEFAULT '';
ALTER TABLE auth DROP CONSTRAINT id_fk_users_id;
ALTER TABLE auth DROP CONSTRAINT auth_pkey;
ALTER TABLE auth DROP CONSTRAINT auth_unique_id;

UPDATE users u SET email = (SELECT email FROM auth WHERE u.id = auth.id);
ALTER TABLE auth DROP COLUMN id;

ALTER TABLE auth ADD PRIMARY KEY (email);

ALTER TABLE teams DROP CONSTRAINT owner_fk_users_id;
ALTER TABLE teams ADD COLUMN owner_email TEXT;
UPDATE teams SET owner_email = (SELECT email FROM users u WHERE owner = u.id);
ALTER TABLE teams ALTER COLUMN owner TYPE TEXT USING owner_email;
ALTER TABLE teams DROP COLUMN owner_email;

ALTER TABLE users DROP CONSTRAINT users_pkey CASCADE;
ALTER TABLE users DROP COLUMN id;
ALTER TABLE users ADD PRIMARY KEY (email);

ALTER TABLE teams ADD CONSTRAINT owner_fk_users_email FOREIGN KEY (owner) REFERENCES users(email) ON DELETE SET NULL;
