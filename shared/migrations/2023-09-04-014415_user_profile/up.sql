CREATE TABLE user_profiles (
    email TEXT PRIMARY KEY UNIQUE NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    country VARCHAR(255),
    school VARCHAR(255) NOT NULL,
    linkedin VARCHAR(255),
    github VARCHAR(255),
    resume_s3_key VARCHAR(255),
    FOREIGN KEY(email) REFERENCES users(email) ON DELETE CASCADE
);