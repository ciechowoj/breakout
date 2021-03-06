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

CREATE OR REPLACE FUNCTION acquire_password_hash(varchar(128))
RETURNS TABLE(name varchar(128), hash varchar(128))
LANGUAGE SQL
AS $$
    UPDATE passwords SET access_time = (SELECT Now()) WHERE name = $1;
    SELECT name, hash FROM passwords WHERE name = $1;
$$;
