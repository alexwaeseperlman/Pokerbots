CREATE TABLE auth (
    email TEXT PRIMARY KEY,
    mangled_password TEXT NOT NULL,
    code TEXT,
    code_expiration TIMESTAMP,
    link TEXT,
    link_expiration TIMESTAMP,
    email_confirmed BOOLEAN NOT NULL
);
