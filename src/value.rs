//! Dynamic JSON value type
//!
//! Provides a flexible representation for JSON values when
//! the structure is not known at compile time.

use std::ops::Index;

/// A dynamic JSON value
#[derive(Debug, Clone, PartialEq, Default)]
pub enum JsonValue {
    /// Null value
    #[default]
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value (fits in i64)
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<JsonValue>),
    /// Object (key-value pairs)
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Check if value is null
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Check if value is a boolean
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// Check if value is a number (integer or float)
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Integer(_) | JsonValue::Float(_))
    }

    /// Check if value is an integer
    #[inline]
    pub fn is_integer(&self) -> bool {
        matches!(self, JsonValue::Integer(_))
    }

    /// Check if value is a float
    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, JsonValue::Float(_))
    }

    /// Check if value is a string
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// Check if value is an array
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Check if value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Get as boolean
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as i64
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Integer(n) => Some(*n),
            JsonValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Get as u64
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Integer(n) if *n >= 0 => Some(*n as u64),
            JsonValue::Float(f) if *f >= 0.0 => Some(*f as u64),
            _ => None,
        }
    }

    /// Get as f64
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Float(f) => Some(*f),
            JsonValue::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Get as string slice
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as array slice
    #[inline]
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as mutable array
    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as object slice
    #[inline]
    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get as mutable object
    #[inline]
    pub fn as_object_mut(&mut self) -> Option<&mut Vec<(String, JsonValue)>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get a value from an object by key
    #[inline]
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Get a value from an array by index
    #[inline]
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// Get mutable reference to a value from an object by key
    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Take ownership of a string value
    #[inline]
    pub fn into_string(self) -> Option<String> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Take ownership of an array value
    #[inline]
    pub fn into_array(self) -> Option<Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Take ownership of an object value
    #[inline]
    pub fn into_object(self) -> Option<Vec<(String, JsonValue)>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

/// Static null value for index out of bounds
static NULL: JsonValue = JsonValue::Null;

impl Index<&str> for JsonValue {
    type Output = JsonValue;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key).unwrap_or(&NULL)
    }
}

impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &Self::Output {
        self.get_index(index).unwrap_or(&NULL)
    }
}

impl From<bool> for JsonValue {
    fn from(v: bool) -> Self {
        JsonValue::Bool(v)
    }
}

impl From<i32> for JsonValue {
    fn from(v: i32) -> Self {
        JsonValue::Integer(v as i64)
    }
}

impl From<i64> for JsonValue {
    fn from(v: i64) -> Self {
        JsonValue::Integer(v)
    }
}

impl From<u32> for JsonValue {
    fn from(v: u32) -> Self {
        JsonValue::Integer(v as i64)
    }
}

impl From<u64> for JsonValue {
    fn from(v: u64) -> Self {
        JsonValue::Integer(v as i64)
    }
}

impl From<f32> for JsonValue {
    fn from(v: f32) -> Self {
        JsonValue::Float(v as f64)
    }
}

impl From<f64> for JsonValue {
    fn from(v: f64) -> Self {
        JsonValue::Float(v)
    }
}

impl From<String> for JsonValue {
    fn from(v: String) -> Self {
        JsonValue::String(v)
    }
}

impl From<&str> for JsonValue {
    fn from(v: &str) -> Self {
        JsonValue::String(v.to_string())
    }
}

impl<T: Into<JsonValue>> From<Vec<T>> for JsonValue {
    fn from(v: Vec<T>) -> Self {
        JsonValue::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<JsonValue>> From<Option<T>> for JsonValue {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => JsonValue::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_methods() {
        assert!(JsonValue::Null.is_null());
        assert!(JsonValue::Bool(true).is_bool());
        assert!(JsonValue::Integer(42).is_integer());
        assert!(JsonValue::Float(3.14).is_float());
        assert!(JsonValue::String("test".into()).is_string());
        assert!(JsonValue::Array(vec![]).is_array());
        assert!(JsonValue::Object(vec![]).is_object());
    }

    #[test]
    fn test_as_methods() {
        assert_eq!(JsonValue::Bool(true).as_bool(), Some(true));
        assert_eq!(JsonValue::Integer(42).as_i64(), Some(42));
        assert_eq!(JsonValue::Float(3.14).as_f64(), Some(3.14));
        assert_eq!(JsonValue::String("test".into()).as_str(), Some("test"));
    }

    #[test]
    fn test_index_object() {
        let obj = JsonValue::Object(vec![
            ("name".into(), JsonValue::String("test".into())),
            ("value".into(), JsonValue::Integer(42)),
        ]);

        assert_eq!(obj["name"].as_str(), Some("test"));
        assert_eq!(obj["value"].as_i64(), Some(42));
        assert!(obj["missing"].is_null());
    }

    #[test]
    fn test_index_array() {
        let arr = JsonValue::Array(vec![
            JsonValue::Integer(1),
            JsonValue::Integer(2),
            JsonValue::Integer(3),
        ]);

        assert_eq!(arr[0].as_i64(), Some(1));
        assert_eq!(arr[1].as_i64(), Some(2));
        assert!(arr[10].is_null());
    }
}
