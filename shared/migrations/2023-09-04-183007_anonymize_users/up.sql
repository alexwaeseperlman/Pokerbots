ALTER TABLE users ADD COLUMN email_hash VARCHAR(255) NOT NULL DEFAULT '';
UPDATE users SET email_hash = MD5(email);