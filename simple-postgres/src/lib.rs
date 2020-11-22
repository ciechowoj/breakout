mod de;
mod error;
mod statement;

pub use de::{Deserializer};

use serde::Deserialize;

trait SqlParam {
    fn r#type(&self) -> libpq::Oid;
    fn value(&self) -> Option<Vec<u8>>;
    fn format(&self) -> libpq::Format;
}

impl SqlParam for &str {
    fn r#type(&self) -> libpq::Oid {
        return libpq::types::CSTRING.oid;
    }

    fn value(&self) -> Option<Vec<u8>> {
        let mut result : Vec<u8> = self.as_bytes().iter().cloned().collect();
        result.push(0);
        return Some(result);
    }

    fn format(&self) -> libpq::Format {
        return libpq::Format::Text;
    }
}

#[macro_export]
macro_rules! impl_sql_param_for_int {
    ($r#type:ty, $oid:expr) => {
        impl SqlParam for $r#type {
            fn r#type(&self) -> libpq::Oid {
                return $oid;
            }

            fn value(&self) -> Option<Vec<u8>> {
                let mut result : Vec<u8> = self.to_string().as_bytes().iter().cloned().collect();
                result.push(0);
                return Some(result);
            }

            fn format(&self) -> libpq::Format {
                return libpq::Format::Text;
            }
        }
    };
}

impl_sql_param_for_int!(u8, libpq::types::INT2.oid);
impl_sql_param_for_int!(u16, libpq::types::INT2.oid);
impl_sql_param_for_int!(u32, libpq::types::INT4.oid);
impl_sql_param_for_int!(u64, libpq::types::INT8.oid);

impl_sql_param_for_int!(i8, libpq::types::INT2.oid);
impl_sql_param_for_int!(i16, libpq::types::INT2.oid);
impl_sql_param_for_int!(i32, libpq::types::INT4.oid);
impl_sql_param_for_int!(i64, libpq::types::INT8.oid);

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Overflow
}

pub fn query<T>(
    connection_string : &str,
    sql_query : &str,
    param_types : &[libpq::Oid],
    param_values: &[Option<Vec<u8>>],
    param_formats: &[libpq::Format]) -> std::result::Result<T, Error>
    where T: for<'de> serde::Deserialize<'de> {
    use libpq::*;

    let connection = Connection::new(connection_string).unwrap();

    let _status = connection.status();

    let _error = connection.error_message();

    // println!("{:?} {:?}", status, error);

    // let result = connection.exec(sql_query);

    let result = connection.exec_params(
        sql_query,
        &param_types,
        &param_values,
        &param_formats,
        Format::Text);

    let _status = result.status();

    // println!("{:?} {:?}", status, result.error_message());

    let result : T = crate::de::from_result(&result).unwrap();

    drop(connection);

    return Ok(result);
}

#[macro_export]
macro_rules! query {
    ($connection_string:expr, $sql_query:expr) => {
        query($connection_string, $sql_query, &[], &[], &[])
    };

    ($connection_string:expr, $sql_query:expr, $($params:expr),*) => {
        query($connection_string, $sql_query, &[ $( $params.r#type() ),* ], &[ $( $params.value() ),* ], &[ $( $params.format() ),* ])
    };
}

#[cfg(test)]
mod tests {
    const CONNECTION_STRING : &'static str = "host=localhost user=testuser dbname=testdb password=password";

    use crate::*;

    #[test]
    fn test_basic_types_params() {
        let value : String = query!(CONNECTION_STRING, "SELECT $1;", "Hello, world!").unwrap();
        assert_eq!("Hello, world!".to_owned() , value);

        let value : (String, String) = query!(CONNECTION_STRING, "SELECT $1, $2;", "Hello!", "World?").unwrap();
        assert_eq!(("Hello!".to_owned(), "World?".to_owned()), value);

        let value : (u8, u16, u32, u64) = query!(CONNECTION_STRING, "SELECT $1, $2, $3, $4;", 8u8, 16u16, 32u32, 64u64).unwrap();
        assert_eq!((8, 16, 32, 64), value);

        let value : (i8, i16, i32, i64) = query!(CONNECTION_STRING, "SELECT $1, $2, $3, $4;", 8i8, 16i16, 32i32, 64i64).unwrap();
        assert_eq!((8, 16, 32, 64), value);

        let value : (bool, bool, bool, bool, bool, bool) = query!(CONNECTION_STRING, "SELECT true, 't', 'true', 'y', 'yes', '1';").unwrap();
        assert_eq!((true, true, true, true, true, true), value);

        let value : (bool, bool, bool, bool, bool, bool) = query!(CONNECTION_STRING, "SELECT false, 'f', 'false', 'n', 'no', '0';").unwrap();
        assert_eq!((false, false, false, false, false, false), value);
    }

    #[test]
    fn test_chrono_params() {
        let value : chrono::DateTime<chrono::Utc> = query!(CONNECTION_STRING, "SELECT $1;", "2014-11-28T21:45:59.324310806+09:00").unwrap();
        assert_eq!(chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339("2014-11-28T21:45:59.324310806+09:00").unwrap().with_timezone(&chrono::Utc), value);
    }

    #[test]
    fn test_uuid_params() {
        let value : uuid::Uuid = query!(CONNECTION_STRING, "SELECT $1;", "936DA01F-9ABD-4D9D-80C7-02AF85C822A8").unwrap();
        assert_eq!(uuid::Uuid::parse_str("936DA01F-9ABD-4D9D-80C7-02AF85C822A8").unwrap(), value);
    }

    #[test]
    fn query_i64() {
        let value : i64 = query!(CONNECTION_STRING, "SELECT * FROM test_i64;").unwrap();
        assert_eq!(42i64, value);
    }

    #[test]
    fn query_vec_i64() {
        let value : Vec<i64> = query!(CONNECTION_STRING, "SELECT * FROM test_vec_i64;").unwrap();
        assert_eq!(vec!(1i64, 2i64, 3i64), value);
    }

    #[test]
    fn query_struct1() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            a: i64,
            b: i64
        };

        let value : Vec<Test> = query!(CONNECTION_STRING, "SELECT * FROM test_struct1;").unwrap();
        assert_eq!(vec!(Test { a: 1, b: 2 }, Test { a: 3, b: 4 }, Test { a: 5, b: 6 }), value);
    }

    #[test]
    fn query_string() {
        let value : String = query!(CONNECTION_STRING, "SELECT * FROM test_string;").unwrap();
        assert_eq!("Hello, world!", value);
    }

    #[test]
    fn query_tuple1() {
        let value : (String, String) = query!(CONNECTION_STRING, "SELECT * FROM test_tuple1;").unwrap();
        assert_eq!(("Hello!".to_owned(), "World!".to_owned()), value);
    }

    #[test]
    fn query_tuple2() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, String);
        let value : TupleStruct = query!(CONNECTION_STRING, "SELECT * FROM test_tuple1;").unwrap();
        assert_eq!(TupleStruct("Hello!".to_owned(), "World!".to_owned()), value);
    }

    #[test]
    fn query_newtype_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct NewtypeStruct(i64);
        let value : NewtypeStruct = query!(CONNECTION_STRING, "SELECT * FROM test_i64;").unwrap();
        assert_eq!(NewtypeStruct(42i64), value);
    }

    /*#[test]
    fn query_multiple_statements() {
        let value : Vec<i64> = query(CONNECTION_STRING, "SELECT * FROM test_vec_i64; SELECT * FROM test_vec_i64;");
        assert_eq!(vec!(1i64, 2i64, 3i64, 1i64, 2i64, 3i64), value);
    }*/


}


