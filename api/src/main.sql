
CREATE TABLE IF NOT EXISTS passwords (
    name varchar(128) PRIMARY KEY,
    hash varchar(128) NOT NULL,
    access_time timestamptz,
    creation_time timestamptz,
    modify_time timestamptz);

CREATE TABLE IF NOT EXISTS high_scores (
    id uuid PRIMARY KEY,
    name varchar(128) NOT NULL,
    score bigint,
    created_time timestamptz);

CREATE OR REPLACE FUNCTION insert_dummy_scores()
RETURNS void
LANGUAGE SQL
AS $$
    INSERT INTO high_scores(id, name, score, created_time)
    VALUES
        ('ed51537e-a654-47c2-95e6-a520ecec5a47', 'First Player', 100, '2020-09-28 11:28:32.258211+02'),
        ('d2bb6f57-2733-4c10-842a-ef956ffd198a', 'Second Player', 90, '2020-09-28 11:28:32.258211+02'),
        ('eaa8e0ae-c434-494a-8151-a9f344878d75', 'Third Player', 80, '2020-09-28 11:28:32.258211+02'),
        ('6c8650e9-74c3-457b-b6fb-08cd756cf3a0', 'Fourth Player', 70, '2020-09-28 11:28:32.258211+02'),
        ('2f640b2a-c87f-45ba-9668-bfc60c63b099', 'Fifth Player', 60, '2020-09-28 11:28:32.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-0a5119b252bc', 'Sixth(0) Player', 50, '2020-09-28 11:28:30.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-1a5119b252bc', 'Sixth(1) Player', 50, '2020-09-28 11:28:31.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-2a5119b252bc', 'Sixth(2) Player', 50, '2020-09-28 11:28:32.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-3a5119b252bc', 'Sixth(3) Player', 50, '2020-09-28 11:28:33.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-4a5119b252bc', 'Sixth(4) Player', 50, '2020-09-28 11:28:33.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-5a5119b252bc', 'Sixth(5) Player', 50, '2020-09-28 11:28:32.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-6a5119b252bc', 'Sixth(6) Player', 50, '2020-09-28 11:28:31.258211+02'),
        ('3dc80d2a-6b6a-4bea-9c6d-7a5119b252bc', 'Sixth(7) Player', 50, '2020-09-28 11:28:30.258211+02'),
        ('2a935a9c-5f0c-4971-aba0-eca8b4e947ed', 'Seventh Player', 40, '2020-09-28 11:28:32.258211+02'),
        ('53acbe82-f6ec-4387-939c-884585175668', 'Eights Player', 30, '2020-09-28 11:28:32.258211+02'),
        ('11597269-1bab-4514-8ec8-b42c06b3f8bf', 'Ninth Player', 20, '2020-09-28 11:28:32.258211+02'),
        ('31865805-de8d-48af-bc12-092ac0ab51a6', 'Tenth Player', 10, '2020-09-28 11:28:32.258211+02')
    ON CONFLICT (id) DO UPDATE
        SET id = excluded.id,
            name = excluded.name,
            score = excluded.score,
            created_time = excluded.created_time
$$;

CREATE OR REPLACE FUNCTION select_adjacent_scores(uuid, bigint)
RETURNS TABLE(index bigint, id uuid, name varchar(128), score bigint)
LANGUAGE SQL
AS $$
    WITH indexed AS (
        SELECT
            ROW_NUMBER() OVER (ORDER BY score DESC, created_time ASC, id ASC) AS index,
            id,
            name,
            score,
            created_time
        FROM high_scores
    ),
        current AS (
            SELECT  index AS current_index,
                    score AS current_score,
                    created_time AS current_created_time
                FROM indexed
                WHERE id = $1
    ),
        collected AS (
        (SELECT index, id, name, score, created_time
            FROM current, indexed
            WHERE score > current_score
            ORDER BY score ASC, created_time DESC
            LIMIT $2)

        UNION

        (SELECT index, id, name, score, created_time
            FROM current, indexed
            WHERE score = current_score AND created_time < current_created_time
            ORDER BY created_time DESC
            LIMIT $2)

        UNION

        (SELECT index, id, name, score, created_time
            FROM current, indexed
            WHERE score = current_score AND created_time >= current_created_time
            ORDER BY created_time ASC
            LIMIT $2)

        UNION

        (SELECT index, id, name, score, created_time
            FROM current, indexed
            WHERE score < current_score
            ORDER BY score DESC, created_time ASC
            LIMIT $2)
    ),
        bounds (lower, upper, minimum, maximum) AS (
        VALUES (
            (SELECT current_index FROM current) - ($2 / 2),
            (SELECT current_index FROM current) - ($2 / 2) + $2,
            (SELECT index FROM collected ORDER BY index ASC LIMIT 1),
            (SELECT index FROM collected ORDER BY index DESC LIMIT 1)
        )
    ),
        effective_bounds (effective_lower, effective_upper) AS (
        VALUES (
            (SELECT LEAST((SELECT lower FROM bounds), (SELECT maximum FROM bounds) - $2 + 1)),
            (SELECT GREATEST((SELECT upper FROM bounds), (SELECT minimum FROM bounds) + $2))
        )
    )
    SELECT index, id, name, score
        FROM collected, effective_bounds
        WHERE effective_lower <= collected.index AND collected.index < effective_upper
        ORDER BY index ASC
$$;

CREATE OR REPLACE FUNCTION upsert_password(varchar(128), varchar(128))
RETURNS void
LANGUAGE SQL
AS $$
    INSERT INTO
        passwords (name, hash, access_time, creation_time, modify_time)
    VALUES
        ($1, $2, (SELECT Now()), (SELECT Now()), (SELECT Now()))
    ON CONFLICT (name) DO UPDATE SET
        name = passwords.name,
        hash = EXCLUDED.hash,
        access_time = passwords.access_time,
        creation_time = passwords.creation_time,
        modify_time = EXCLUDED.modify_time;
$$;

CREATE OR REPLACE FUNCTION acquire_password_hash(varchar(128))
RETURNS TABLE(name varchar(128), hash varchar(128))
LANGUAGE SQL
AS $$
    UPDATE passwords SET access_time = (SELECT Now()) WHERE name = $1;
    SELECT name, hash FROM passwords WHERE name = $1;
$$;
