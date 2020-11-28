
const CONNECTION_STRING : &'static str = "host=localhost user=testuser dbname=testdb password=password";

use simple_postgres::*;

use serde::Deserialize;

#[test]
fn test_basic_types_params() {
    let connection = Connection::new(CONNECTION_STRING);

    let _value : () = query!(connection, "SELECT $1;", ()).unwrap();
    let _value : () = query!(connection, "SELECT NULL;", ()).unwrap();
    let _value : () = query!(connection, "SELECT value FROM test_i64 WHERE value != 42;", ()).unwrap();

    let value : String = query!(connection, "SELECT $1;", "Hello, world!").unwrap();
    assert_eq!("Hello, world!".to_owned() , value);

    let value : (String, String) = query!(connection, "SELECT $1, $2;", "Hello!", "World?").unwrap();
    assert_eq!(("Hello!".to_owned(), "World?".to_owned()), value);

    let value : (u8, u16, u32, u64) = query!(connection, "SELECT $1, $2, $3, $4;", 8u8, 16u16, 32u32, 64u64).unwrap();
    assert_eq!((8, 16, 32, 64), value);

    let value : (i8, i16, i32, i64) = query!(connection, "SELECT $1, $2, $3, $4;", 8i8, 16i16, 32i32, 64i64).unwrap();
    assert_eq!((8, 16, 32, 64), value);

    let value : (bool, bool, bool, bool, bool, bool) = query!(connection, "SELECT true, 't', 'true', 'y', 'yes', '1';").unwrap();
    assert_eq!((true, true, true, true, true, true), value);

    let value : (bool, bool, bool, bool, bool, bool) = query!(connection, "SELECT false, 'f', 'false', 'n', 'no', '0';").unwrap();
    assert_eq!((false, false, false, false, false, false), value);
}

#[test]
fn test_chrono_params() {
    let test_date_time = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339("2014-11-28T21:45:59.324311+09:00").unwrap().with_timezone(&chrono::Utc);
    let connection = Connection::new(CONNECTION_STRING);
    let value : chrono::DateTime<chrono::Utc> = query!(connection, "SELECT $1;", test_date_time).unwrap();
    assert_eq!(test_date_time, value);
}

#[test]
fn test_uuid_params() {
    let connection = Connection::new(CONNECTION_STRING);
    let value : uuid::Uuid = query!(connection, "SELECT $1;", uuid::Uuid::parse_str("936DA01F-9ABD-4D9D-80C7-02AF85C822A8").unwrap()).unwrap();
    assert_eq!(uuid::Uuid::parse_str("936DA01F-9ABD-4D9D-80C7-02AF85C822A8").unwrap(), value);
}

#[test]
fn query_i64() {
    let connection = Connection::new(CONNECTION_STRING);
    let value : i64 = query!(connection, "SELECT * FROM test_i64;").unwrap();
    assert_eq!(42i64, value);
}

#[test]
fn query_vec_i64() {
    let connection = Connection::new(CONNECTION_STRING);
    let value : Vec<i64> = query!(connection, "SELECT * FROM test_vec_i64;").unwrap();
    assert_eq!(vec!(1i64, 2i64, 3i64), value);
}

#[test]
fn query_struct1() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Test {
        a: i64,
        b: i64
    };

    let connection = Connection::new(CONNECTION_STRING);
    let value : Vec<Test> = query!(connection, "SELECT * FROM test_struct1;").unwrap();
    assert_eq!(vec!(Test { a: 1, b: 2 }, Test { a: 3, b: 4 }, Test { a: 5, b: 6 }), value);
}

#[test]
fn query_string() {
    let connection = Connection::new(CONNECTION_STRING);
    let value : String = query!(connection, "SELECT * FROM test_string;").unwrap();
    assert_eq!("Hello, world!", value);
}

#[test]
fn query_tuple1() {
    let connection = Connection::new(CONNECTION_STRING);
    let value : (String, String) = query!(connection, "SELECT * FROM test_tuple1;").unwrap();
    assert_eq!(("Hello!".to_owned(), "World!".to_owned()), value);
}

#[test]
fn query_tuple2() {
    let connection = Connection::new(CONNECTION_STRING);
    #[derive(Debug, Deserialize, PartialEq)]
    struct TupleStruct(String, String);
    let value : TupleStruct = query!(connection, "SELECT * FROM test_tuple1;").unwrap();
    assert_eq!(TupleStruct("Hello!".to_owned(), "World!".to_owned()), value);
}

#[test]
fn query_newtype_struct() {
    let connection = Connection::new(CONNECTION_STRING);
    #[derive(Debug, Deserialize, PartialEq)]
    struct NewtypeStruct(i64);
    let value : NewtypeStruct = query!(connection, "SELECT * FROM test_i64;").unwrap();
    assert_eq!(NewtypeStruct(42i64), value);
}

#[test]
fn unique_constraint() {
    let connection = Connection::new(CONNECTION_STRING);
    #[derive(Debug, Deserialize, PartialEq)]
    struct NewtypeStruct(i64);
    let value : std::result::Result<(), Error> = query!(connection, "INSERT INTO unique_constraint VALUES (42);");

    assert_eq!(SqlState::UniqueViolation, value.err().unwrap().sql_state);
}

/*#[test]
fn query_multiple_statements() {
    let value : Vec<i64> = query(CONNECTION_STRING, "SELECT * FROM test_vec_i64; SELECT * FROM test_vec_i64;");
    assert_eq!(vec!(1i64, 2i64, 3i64, 1i64, 2i64, 3i64), value);
}*/

