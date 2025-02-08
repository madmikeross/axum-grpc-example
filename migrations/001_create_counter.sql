CREATE TABLE IF NOT EXISTS counter
(
    id    SERIAL PRIMARY KEY,
    count INTEGER NOT NULL DEFAULT 0
);

-- Insert a default record
INSERT INTO counter (count)
SELECT 0
WHERE NOT EXISTS (SELECT 1 FROM counter);