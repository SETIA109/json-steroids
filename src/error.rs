//! Error types for JSON parsing and serialization

use std::fmt;

/// Result type alias for JSON operations
pub type Result<T> = std::result::Result<T, JsonError>;

/// Errors that can occur during JSON parsing or serialization
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// Unexpected end of input
    UnexpectedEnd,
    /// Unexpected character encountered
    UnexpectedChar(char, usize),
    /// Expected a specific character
    ExpectedChar(char, usize),
    /// Expected a specific token
    ExpectedToken(&'static str, usize),
    /// Invalid number format
    InvalidNumber(usize),
    /// Invalid escape sequence
    InvalidEscape(usize),
    /// Invalid Unicode escape
    InvalidUnicode(usize),
    /// Invalid UTF-8 encoding
    InvalidUtf8,
    /// Missing required field during deserialization
    MissingField(String),
    /// Unknown enum variant
    UnknownVariant(String),
    /// Type mismatch during deserialization
    TypeMismatch { expected: &'static str, found: &'static str, position: usize },
    /// Nesting too deep
    NestingTooDeep(usize),
    /// Custom error message
    Custom(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedEnd => write!(f, "unexpected end of JSON input"),
            JsonError::UnexpectedChar(c, pos) => write!(f, "unexpected character '{}' at position {}", c, pos),
            JsonError::ExpectedChar(c, pos) => write!(f, "expected '{}' at position {}", c, pos),
            JsonError::ExpectedToken(token, pos) => write!(f, "expected {} at position {}", token, pos),
            JsonError::InvalidNumber(pos) => write!(f, "invalid number at position {}", pos),
            JsonError::InvalidEscape(pos) => write!(f, "invalid escape sequence at position {}", pos),
            JsonError::InvalidUnicode(pos) => write!(f, "invalid unicode escape at position {}", pos),
            JsonError::InvalidUtf8 => write!(f, "invalid UTF-8 encoding"),
            JsonError::MissingField(field) => write!(f, "missing required field: {}", field),
            JsonError::UnknownVariant(variant) => write!(f, "unknown variant: {}", variant),
            JsonError::TypeMismatch { expected, found, position } => {
                write!(f, "type mismatch at position {}: expected {}, found {}", position, expected, found)
            }
            JsonError::NestingTooDeep(depth) => write!(f, "nesting too deep: {} levels", depth),
            JsonError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for JsonError {}

