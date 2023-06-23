CREATE TABLE bots (
    id SERIAL PRIMARY KEY NOT NULL,
    team INTEGER NOT NULL,
    name TEXT NOT NULL check(
        length(description) <= 1000
    ),
    description TEXT check(
        length(description) <= 1000
    ),
    score REAL NOT NULL DEFAULT 0,
    created BIGINT NOT NULL DEFAULT extract(epoch from CURRENT_TIMESTAMP),
    uploaded_by TEXT NOT NULL
);