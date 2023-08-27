ALTER TABLE auth
RENAME COLUMN email_verification_link TO code;

ALTER TABLE auth
RENAME COLUMN email_verification_link_expiration TO code_expiration;

ALTER TABLE auth
RENAME COLUMN password_reset_link TO link;

ALTER TABLE auth
RENAME COLUMN password_reset_link_expiration TO link_expiration;
