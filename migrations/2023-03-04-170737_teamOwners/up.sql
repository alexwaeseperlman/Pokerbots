ALTER TABLE teams
ADD owner TEXT NOT NULL;

-- This is safe because these teams are inaccessible
DELETE FROM teams WHERE owner IS NULL;

ALTER TABLE teams
ADD CONSTRAINT owner_isUserId
FOREIGN KEY (owner) REFERENCES users(email);

