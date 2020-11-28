mod de;
mod error;
pub mod statement;

pub use de::{Deserializer};

pub use error::*;

pub trait SqlParam {
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

impl SqlParam for uuid::Uuid {
    fn r#type(&self) -> libpq::Oid {
        return libpq::types::UUID.oid;
    }

    fn value(&self) -> Option<Vec<u8>> {
        let mut buffer = uuid::Uuid::encode_buffer();
        let string = self.to_hyphenated().encode_lower(&mut buffer);
        let mut result : Vec<u8> = string.as_bytes().iter().cloned().collect();
        result.push(0);
        return Some(result);
    }

    fn format(&self) -> libpq::Format {
        return libpq::Format::Text;
    }
}

impl SqlParam for chrono::DateTime<chrono::Utc> {
    fn r#type(&self) -> libpq::Oid {
        return libpq::types::TIMESTAMPTZ.oid;
    }

    fn value(&self) -> Option<Vec<u8>> {
        let string = self.to_rfc3339();
        let mut result : Vec<u8> = string.as_bytes().iter().cloned().collect();
        result.push(0);
        return Some(result);
    }

    fn format(&self) -> libpq::Format {
        return libpq::Format::Text;
    }
}

impl SqlParam for () {
    fn r#type(&self) -> libpq::Oid {
        return libpq::types::ANY.oid;
    }

    fn value(&self) -> Option<Vec<u8>> {
        return None;
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

pub struct Connection {
    connection : libpq::Connection
}

impl Connection {
    pub fn new(connection_string : &str) -> Self {
        return Connection {
            connection : libpq::Connection::new(connection_string).unwrap()
        }
    }

    pub fn query<T>(
        &self,
        sql_query : &str,
        param_types : &[libpq::Oid],
        param_values: &[Option<Vec<u8>>],
        param_formats: &[libpq::Format]) -> std::result::Result<T, Error>
        where T: for<'de> serde::Deserialize<'de> {
        use libpq::*;

        let _status = self.connection.status();

        let _error = self.connection.error_message();

        let result = self.connection.exec_params(
            sql_query,
            &param_types,
            &param_values,
            &param_formats,
            Format::Text);

        let _status = result.status();

        let sql_state = result.error_field(result::ErrorField::Sqlstate);

        if let Some(sql_state) = sql_state {
            if sql_state_from_code(sql_state) != SqlState::SuccessfulCompletion {

                let message_primary = result.error_field(result::ErrorField::MessagePrimary).unwrap();
                let message_detail = result.error_field(result::ErrorField::MessageDetail);

                let details = if let Some(message_detail) = message_detail {
                    format!("{}\n{}", message_primary, message_detail)
                }
                else {
                    message_primary.to_string()
                };

                let error = Error {
                    sql_state: sql_state_from_code(sql_state),
                    details: details
                };

                return Err(error);
            }
        }

        let result : T = crate::de::from_result(&result).unwrap();

        return Ok(result);
    }
}

#[macro_export]
macro_rules! query {
    ($connection:expr, $sql_query:expr) => {
        $connection.query($sql_query, &[], &[], &[])
    };

    ($connection:expr, $sql_query:expr, $($params:expr),*) => {
        $connection.query($sql_query, &[ $( SqlParam::r#type(&$params) ),* ], &[ $( SqlParam::value(&$params) ),* ], &[ $( SqlParam::format(&$params) ),* ])
    };
}
