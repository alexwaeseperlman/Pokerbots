ALTER TABLE auth
ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE auth
SET is_admin = users.is_admin
FROM users
WHERE auth.email = users.email;

ALTER TABLE users
DROP COLUMN is_admin;
