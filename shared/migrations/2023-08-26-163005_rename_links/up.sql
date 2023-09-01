ALTER TABLE auth
RENAME COLUMN code TO email_verification_link;

ALTER TABLE auth
RENAME COLUMN code_expiration TO email_verification_link_expiration;

ALTER TABLE auth
RENAME COLUMN link TO password_reset_link;

ALTER TABLE auth
RENAME COLUMN link_expiration TO password_reset_link_expiration;
