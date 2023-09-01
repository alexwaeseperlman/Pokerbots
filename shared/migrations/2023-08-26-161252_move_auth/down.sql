ALTER TABLE users
ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE users
SET is_admin = auth.is_admin
FROM auth
WHERE users.email = auth.email;

ALTER TABLE auth
DROP COLUMN is_admin;

