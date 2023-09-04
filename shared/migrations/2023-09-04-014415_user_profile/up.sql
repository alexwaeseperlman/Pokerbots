CREATE TABLE user_profile (
    user TEXT PRIMARY KEY NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    country VARCHAR(255),
    school VARCHAR(255) NOT NULL,
    linkedin VARCHAR(255),
    github VARCHAR(255),
    resume_s3_key VARCHAR(255),
);