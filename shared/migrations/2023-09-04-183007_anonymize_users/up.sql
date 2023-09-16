ALTER TABLE users ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE users ADD CONSTRAINT users_unique_id UNIQUE (id);
ALTER TABLE users DROP CONSTRAINT users_pkey CASCADE;
ALTER TABLE users ADD PRIMARY KEY (id);

ALTER TABLE teams ADD COLUMN owner_id UUID;
UPDATE teams SET owner_id = (SELECT id FROM users WHERE email = owner);
ALTER TABLE teams ALTER COLUMN owner TYPE UUID USING owner_id;
ALTER TABLE teams DROP COLUMN owner_id;
ALTER TABLE teams ADD CONSTRAINT owner_fk_users_id FOREIGN KEY (owner) REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE auth DROP CONSTRAINT auth_pkey CASCADE;

ALTER TABLE auth ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid();
UPDATE auth SET id = (SELECT id FROM users WHERE email = auth.email);

ALTER TABLE auth ADD CONSTRAINT auth_unique_id UNIQUE (id);
ALTER TABLE auth ADD PRIMARY KEY (id);
ALTER TABLE auth ADD CONSTRAINT id_fk_users_id FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE users DROP COLUMN email CASCADE;

ALTER TABLE bots ADD COLUMN uploaded_by_id UUID;
UPDATE bots SET uploaded_by_id = (SELECT id FROM auth WHERE email = uploaded_by);
ALTER TABLE bots ALTER COLUMN uploaded_by TYPE UUID USING uploaded_by_id;
ALTER TABLE bots DROP COLUMN uploaded_by_id;
ALTER TABLE bots ADD CONSTRAINT bots_uploaded_by_fk_auth_id FOREIGN KEY (uploaded_by) REFERENCES auth(id) ON DELETE SET NULL;

ALTER TABLE user_profiles ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid();
UPDATE user_profiles profiles SET id = (SELECT id FROM auth a WHERE a.email = profiles.email);

ALTER TABLE user_profiles ADD CONSTRAINT user_profiles_unique_id UNIQUE (id);
ALTER TABLE user_profiles DROP CONSTRAINT user_profiles_pkey CASCADE;
ALTER TABLE user_profiles ADD PRIMARY KEY (id);
ALTER TABLE user_profiles ADD CONSTRAINT user_profiles_fk_auth_id FOREIGN KEY (id) REFERENCES auth(id) ON DELETE CASCADE;