//! Serialization and deserialization traits
//!
//! These traits are implemented for primitive types and can be derived
//! for custom structs and enums using the derive macros.

use crate::error::Result;
use crate::parser::{JsonParser, ParseInt};
use crate::writer::{JsonWriter, Writer};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Trait for types that can be serialized to JSON
pub trait JsonSerialize {
    /// Serialize this value to the given writer
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>);
}

/// Trait for types that can be deserialized from JSON
pub trait JsonDeserialize: Sized {
    /// Deserialize a value from the given parser
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self>;
}

// ============ Primitive implementations ============

impl JsonSerialize for bool {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.write_bool(*self);
    }
}

impl JsonDeserialize for bool {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.parse_bool()
    }
}

// Signed integers
macro_rules! impl_json_signed {
    ($($ty:ty),*) => {
        $(
            impl JsonSerialize for $ty {
                #[inline]
                fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
                    writer.write_i64(*self as i64);
                }
            }

            impl JsonDeserialize for $ty {
                #[inline]
                fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self>
                where Self: ParseInt {
                    parser.parse_integer::<Self>()
                }
            }
        )*
    };
}

impl_json_signed!(i8, i16, i32, i64, isize);

// Unsigned integers
macro_rules! impl_json_unsigned {
    ($($ty:ty),*) => {
        $(
            impl JsonSerialize for $ty {
                #[inline]
                fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
                    writer.write_u64(*self as u64);
                }
            }

            impl JsonDeserialize for $ty {
                #[inline]
                fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self>
                where Self: ParseInt {
                    parser.parse_integer::<Self>()
                }
            }
        )*
    };
}

impl_json_unsigned!(u8, u16, u32, u64, usize);

// Floats
macro_rules! impl_json_float {
    ($($ty:ty),*) => {
        $(
            impl JsonSerialize for $ty {
                #[inline]
                fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
                    writer.write_f64(*self as f64);
                }
            }

            impl JsonDeserialize for $ty {
                #[inline]
                fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
                    parser.parse_float()
                }
            }
        )*
    };
}

impl_json_float!(f32, f64);

// String types
impl JsonSerialize for String {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.write_string(self);
    }
}

impl JsonDeserialize for String {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.parse_string().map(|s| s.into_owned())
    }
}

impl JsonSerialize for str {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.write_string(self);
    }
}

impl<'a> JsonSerialize for Cow<'a, str> {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.write_string(self);
    }
}

impl<'a> JsonDeserialize for Cow<'a, str> {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.parse_string().map(|s| Cow::Owned(s.into_owned()))
    }
}

// Option
impl<T: JsonSerialize> JsonSerialize for Option<T> {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        match self {
            Some(value) => value.json_serialize(writer),
            None => writer.write_null(),
        }
    }
}

impl<T: JsonDeserialize> JsonDeserialize for Option<T> {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.skip_whitespace_pub();
        if parser.peek_is_null() {
            parser.expect_null()?;
            Ok(None)
        } else {
            T::json_deserialize(parser).map(Some)
        }
    }
}

// Vec
impl<T: JsonSerialize> JsonSerialize for Vec<T> {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.begin_array();
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                writer.write_comma();
            }
            item.json_serialize(writer);
        }
        writer.end_array();
    }
}

impl<T: JsonDeserialize> JsonDeserialize for Vec<T> {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.expect_array_start()?;
        // Start with a reasonable capacity to avoid repeated reallocations
        let mut result = Vec::with_capacity(8);

        let mut first = true;
        loop {
            if !parser.has_next_array_element_or_first(first)? {
                break;
            }
            first = false;
            result.push(T::json_deserialize(parser)?);
        }

        parser.expect_array_end()?;
        Ok(result)
    }
}

// Slice (serialize only)
impl<T: JsonSerialize> JsonSerialize for [T] {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.begin_array();
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                writer.write_comma();
            }
            item.json_serialize(writer);
        }
        writer.end_array();
    }
}

// Arrays
impl<T: JsonSerialize, const N: usize> JsonSerialize for [T; N] {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.begin_array();
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                writer.write_comma();
            }
            item.json_serialize(writer);
        }
        writer.end_array();
    }
}

// HashMap
impl<K: AsRef<str>, V: JsonSerialize> JsonSerialize for HashMap<K, V> {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.begin_object();
        for (i, (key, value)) in self.iter().enumerate() {
            if i > 0 {
                writer.write_comma();
            }
            writer.write_key(key.as_ref());
            value.json_serialize(writer);
        }
        writer.end_object();
    }
}

impl<K: JsonDeserialize + Eq + Hash, V: JsonDeserialize> JsonDeserialize for HashMap<K, V>
where
    K: From<String>,
{
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.expect_object_start()?;
        let mut result = HashMap::new();

        while let Some(key) = parser.next_object_key()? {
            let value = V::json_deserialize(parser)?;
            result.insert(K::from(key.into_owned()), value);
        }

        Ok(result)
    }
}

// BTreeMap
impl<K: AsRef<str>, V: JsonSerialize> JsonSerialize for BTreeMap<K, V> {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.begin_object();
        for (i, (key, value)) in self.iter().enumerate() {
            if i > 0 {
                writer.write_comma();
            }
            writer.write_key(key.as_ref());
            value.json_serialize(writer);
        }
        writer.end_object();
    }
}

impl<K: JsonDeserialize + Ord, V: JsonDeserialize> JsonDeserialize for BTreeMap<K, V>
where
    K: From<String>,
{
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.expect_object_start()?;
        let mut result = BTreeMap::new();

        while let Some(key) = parser.next_object_key()? {
            let value = V::json_deserialize(parser)?;
            result.insert(K::from(key.into_owned()), value);
        }

        Ok(result)
    }
}

// Box
impl<T: JsonSerialize + ?Sized> JsonSerialize for Box<T> {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        (**self).json_serialize(writer);
    }
}

impl<T: JsonDeserialize> JsonDeserialize for Box<T> {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        T::json_deserialize(parser).map(Box::new)
    }
}

// References (serialize only) - blanket impl for all references
impl<T: JsonSerialize + ?Sized> JsonSerialize for &T {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        (**self).json_serialize(writer);
    }
}

impl<T: JsonSerialize + ?Sized> JsonSerialize for &mut T {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        (**self).json_serialize(writer);
    }
}

// Unit type
impl JsonSerialize for () {
    #[inline]
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        writer.write_null();
    }
}

impl JsonDeserialize for () {
    #[inline]
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.expect_null()?;
        Ok(())
    }
}

// Tuples
macro_rules! impl_tuple {
    ($($idx:tt $T:ident),+) => {
        impl<$($T: JsonSerialize),+> JsonSerialize for ($($T,)+) {
            #[inline]
            #[allow(unused_assignments)]
            fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
                writer.begin_array();
                let mut first = true;
                $(
                    if !first {
                        writer.write_comma();
                    }
                    first = false;
                    self.$idx.json_serialize(writer);
                )+
                writer.end_array();
            }
        }

        impl<$($T: JsonDeserialize),+> JsonDeserialize for ($($T,)+) {
            #[inline]
            fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
                parser.expect_array_start()?;
                let result = ($(
                    {
                        if $idx > 0 {
                            parser.expect_comma()?;
                        }
                        $T::json_deserialize(parser)?
                    },
                )+);
                parser.expect_array_end()?;
                Ok(result)
            }
        }
    };
}

impl_tuple!(0 T0);
impl_tuple!(0 T0, 1 T1);
impl_tuple!(0 T0, 1 T1, 2 T2);
impl_tuple!(0 T0, 1 T1, 2 T2, 3 T3);
impl_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4);
impl_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5);
impl_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6);
impl_tuple!(0 T0, 1 T1, 2 T2, 3 T3, 4 T4, 5 T5, 6 T6, 7 T7);

// JsonValue
impl JsonSerialize for crate::JsonValue {
    fn json_serialize<W: Writer>(&self, writer: &mut JsonWriter<W>) {
        use crate::JsonValue;
        match self {
            JsonValue::Null => writer.write_null(),
            JsonValue::Bool(b) => writer.write_bool(*b),
            JsonValue::Integer(n) => writer.write_i64(*n),
            JsonValue::Float(f) => writer.write_f64(*f),
            JsonValue::String(s) => writer.write_string(s),
            JsonValue::Array(arr) => {
                writer.begin_array();
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        writer.write_comma();
                    }
                    item.json_serialize(writer);
                }
                writer.end_array();
            }
            JsonValue::Object(obj) => {
                writer.begin_object();
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        writer.write_comma();
                    }
                    writer.write_key(key);
                    value.json_serialize(writer);
                }
                writer.end_object();
            }
        }
    }
}

impl JsonDeserialize for crate::JsonValue {
    fn json_deserialize(parser: &mut JsonParser<'_>) -> Result<Self> {
        parser.parse_value()
    }
}
