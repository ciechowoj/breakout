CREATE TABLE test_i64 (value int);
INSERT INTO test_i64 VALUES (42);

CREATE TABLE test_vec_i64 (value int);
INSERT INTO test_vec_i64 VALUES (1), (2), (3);

CREATE TABLE test_struct1 (a int, b int);
INSERT INTO test_struct1 VALUES (1, 2), (3, 4), (5, 6);

CREATE TABLE test_string (value  VARCHAR(128));
INSERT INTO test_string VALUES ('Hello, world!');

CREATE TABLE test_tuple1 (a  VARCHAR(128), b VARCHAR(128));
INSERT INTO test_tuple1 VALUES ('Hello!', 'World!');

CREATE TABLE unique_constraint (id int PRIMARY KEY);
INSERT INTO unique_constraint VALUES (42);