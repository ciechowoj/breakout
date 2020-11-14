CREATE TABLE IF NOT EXISTS passwords (
    name varchar(128) PRIMARY KEY,
    hash varchar(128) NOT NULL,
    access_time timestamptz,
    creation_time timestamptz,
    modify_time timestamptz);

INSERT INTO
    passwords (name, hash, access_time, creation_time, modify_time)
VALUES
    ('root', '{{hash}}', (SELECT Now()), (SELECT Now()), (SELECT Now()))
ON CONFLICT (name) DO UPDATE SET
    name = passwords.name,
    hash = EXCLUDED.hash,
    access_time = passwords.access_time,
    creation_time = passwords.creation_time,
    modify_time = EXCLUDED.modify_time;