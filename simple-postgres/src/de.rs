
use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, /* EnumAccess,*/ /* IntoDeserializer, */ MapAccess, SeqAccess,
    /* VariantAccess,*/ Visitor,
};

use crate::error::{SqlState, Error, Result};

pub struct Deserializer<'de> {
    row_index: usize,
    col_index: usize,
    result: &'de libpq::result::Result,
}

impl<'de> Deserializer<'de> {
    pub fn from_result(result: &'de libpq::result::Result) -> Self {
        Deserializer { row_index: 0, col_index: 0, result: result }
    }
}

pub fn from_result<T>(result: &libpq::result::Result) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_result(result);
    return Ok(T::deserialize(&mut deserializer)?);
}

impl<'de> Deserializer<'de> {
    // Look at the first character in the input without consuming it.
    // fn peek_char(&mut self) -> Result<char> {
    //     self.input.chars().next().ok_or(Error::Eof)
    // }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::from_sql_state(SqlState::DeserializeErrorAnyNotSupported))
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let fformat = self.result.field_format(self.col_index);

        if fformat == libpq::Format::Text {
            let value = self.result.value(self.row_index, self.col_index).unwrap();
            let value = std::str::from_utf8(value).unwrap();

            let value = match value {
                "true" => true,
                "t" => true,
                "yes" => true,
                "y" => true,
                "1" => true,
                "false" => false,
                "f" => false,
                "no" => false,
                "n" => false,
                "0" => false,
                _ => return Err(Error::from_sql_state(SqlState::DeserializeErrorExpectedBoolean))
            };

            visitor.visit_bool(value)
        }
        else {
            unimplemented!()
        }
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let fformat = self.result.field_format(self.col_index);

        if fformat == libpq::Format::Text {
            let value = self.result.value(self.row_index, self.col_index).unwrap();
            let value = std::str::from_utf8(value).unwrap();
            let value : i64 = value.parse().unwrap();
            visitor.visit_i64(value)
        }
        else {
            unimplemented!()
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let fformat = self.result.field_format(self.col_index);

        if fformat == libpq::Format::Text {
            let value = self.result.value(self.row_index, self.col_index);

            match value {
                Some(value) => {
                    let value = std::str::from_utf8(value).unwrap();

                    if self.result.field_type(self.col_index) == libpq::types::TIMESTAMPTZ.oid {
                        if let Ok(datetime) = chrono::DateTime::parse_from_str(value, " %Y-%m-%d %H:%M:%S%.f%#z") {
                            return visitor.visit_string(datetime.to_rfc3339());
                        }
                        else {
                            use serde::de::Error;
                            return Err(Error::custom("invalid date format"));
                        }
                    }
                    else {
                        return visitor.visit_string(value.to_owned());
                    }
                },
                None => {
                    return Err(Error::from_sql_state(SqlState::DeserializeErrorUnexpectedNull));
                }
            }
        }
        else {
            unimplemented!()
        }
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // if self.input.starts_with("null") {
        //     self.input = &self.input["null".len()..];
        //     visitor.visit_none()
        // } else {
        //     visitor.visit_some(self)
        // }

        unimplemented!()
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.result.nfields() == 0 || self.result.ntuples() == 0 {
            visitor.visit_unit()
        }
        else {
            let fformat = self.result.field_format(self.col_index);

            if fformat == libpq::Format::Text {
                let value = self.result.value(self.row_index, self.col_index);

                if value == None {
                    visitor.visit_unit()
                }
                else {
                    return Err(Error::from_sql_state(SqlState::DeserializeErrorExpectedNull));
                }
            }
            else {
                unimplemented!()
            }
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(
        mut self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // [TODO] Check _len before parsing.
        let value = visitor.visit_seq(TupleIterator::new(&mut self))?;
        return Ok(value);
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(RowIterator::new(&mut self))?;
        return Ok(value);
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(mut self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // [TODO] Check _len before parsing.
        let value = visitor.visit_seq(TupleIterator::new(&mut self))?;
        return Ok(value);
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        mut self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // [TODO] Check _len before parsing.
        let value = visitor.visit_seq(TupleIterator::new(&mut self))?;
        return Ok(value);
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.col_index = 0;
        let value = visitor.visit_map(ColumnIterator::new(&mut self))?;
        return Ok(value);
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // if self.peek_char()? == '"' {
        //     // Visit a unit variant.
        //     visitor.visit_enum(self.parse_string()?.into_deserializer())
        // } else if self.next_char()? == '{' {
        //     // Visit a newtype variant, tuple variant, or struct variant.
        //     let value = visitor.visit_enum(Enum::new(self))?;
        //     // Parse the matching close brace.
        //     if self.next_char()? == '}' {
        //         Ok(value)
        //     } else {
        //         Err(Error::ExpectedMapEnd)
        //     }
        // } else {
        //     Err(Error::ExpectedEnum)
        // }

        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        return visitor.visit_string(self.result.field_name(self.col_index).unwrap());
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        use serde::de::Error;
        let message = "Postgres does not support Deserializer::deserialize_ignored_any";
        Err(Error::custom(message))
    }
}

struct RowIterator<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>
}

impl<'a, 'de> RowIterator<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        RowIterator {
            de
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for RowIterator<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        if self.de.row_index < self.de.result.ntuples() {
            let result = seed.deserialize(&mut *self.de).map(Some);
            self.de.row_index += 1;
            return result;
        }
        else {
            return Ok(None);
        }
    }
}

struct ColumnIterator<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>
}

impl<'a, 'de> ColumnIterator<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        ColumnIterator {
            de
        }
    }
}

impl<'de, 'a> MapAccess<'de> for ColumnIterator<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where K: DeserializeSeed<'de>,
    {
        if self.de.col_index < self.de.result.nfields() {
            return seed.deserialize(&mut *self.de).map(Some)
        }
        else {
            return Ok(None);
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where V: DeserializeSeed<'de>,
    {
        let result = seed.deserialize(&mut *self.de);
        self.de.col_index += 1;
        return result;
    }
}

struct TupleIterator<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>
}

impl<'a, 'de> TupleIterator<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        TupleIterator {
            de
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for TupleIterator<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        if self.de.col_index < self.de.result.nfields() {
            let result = seed.deserialize(&mut *self.de).map(Some);
            self.de.col_index += 1;
            return result;
        }
        else {
            return Ok(None);
        }
    }
}

/*

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        let val = seed.deserialize(&mut *self.de)?;
        // Parse the colon separating map key from value.
        if self.de.next_char()? == ':' {
            Ok((val, self))
        } else {
            Err(Error::ExpectedMapColon)
        }
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}
*/
