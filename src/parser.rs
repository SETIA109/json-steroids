//! High-performance JSON parser with zero-copy support
//!
//! This parser is optimized for speed with:
//! - Zero-copy string parsing when no escapes are present
//! - Minimal allocations
//! - Efficient number parsing
//! - SIMD-friendly byte scanning

use crate::error::{JsonError, Result};
use crate::value::JsonValue;
use std::borrow::Cow;

/// Maximum nesting depth to prevent stack overflow
const MAX_DEPTH: usize = 128;

/// Lookup table for whitespace bytes (space, tab, newline, carriage-return)
static WS: [bool; 256] = {
    let mut t = [false; 256];
    t[b' ' as usize] = true;
    t[b'\t' as usize] = true;
    t[b'\n' as usize] = true;
    t[b'\r' as usize] = true;
    t
};

/// Sealed helper trait for fast integer parsing without going through `str::parse`.
/// `from_parts(negative, abs_value)` converts the sign + magnitude to the concrete type.
pub trait ParseInt: Sized {
    fn from_parts(negative: bool, value: u64) -> Option<Self>;
}

macro_rules! impl_parse_int_signed {
    ($($t:ty),*) => {
        $(
            impl ParseInt for $t {
                #[inline]
                fn from_parts(negative: bool, value: u64) -> Option<Self> {
                    if negative {
                        // i64::MIN has magnitude 2^63 which does NOT fit in a positive i64.
                        // Use wrapping_neg on i64 to handle it correctly, then range-check.
                        let neg = (value as i64).wrapping_neg();
                        // Reject if value was 0 (would give 0, not negative) or overflowed
                        // past i64::MIN (value > 2^63 → wrapping_neg gives positive).
                        if value != 0 && neg >= 0 {
                            return None; // magnitude too large
                        }
                        <$t>::try_from(neg).ok()
                    } else {
                        <$t>::try_from(value).ok()
                    }
                }
            }
        )*
    };
}

macro_rules! impl_parse_int_unsigned {
    ($($t:ty),*) => {
        $(
            impl ParseInt for $t {
                #[inline]
                fn from_parts(negative: bool, value: u64) -> Option<Self> {
                    if negative { return None; }
                    <$t>::try_from(value).ok()
                }
            }
        )*
    };
}

impl_parse_int_signed!(i8, i16, i32, i64, isize);
impl_parse_int_unsigned!(u8, u16, u32, u64, usize);

/// High-performance JSON parser
pub struct JsonParser<'a> {
    /// Input bytes
    input: &'a [u8],
    /// Current position
    pos: usize,
    /// Current nesting depth
    depth: usize,
    /// Total length of input (cached for efficiency)
    len: usize,
}

impl<'a> JsonParser<'a> {
    /// Create a new parser from a string slice
    #[inline]
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            depth: 0,
            len: input.len(),
        }
    }

    /// Create a new parser from bytes
    #[inline]
    pub fn from_bytes(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            depth: 0,
            len: input.len(),
        }
    }

    /// Get current position
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Check if we've reached the end
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pos >= self.len
    }

    /// Peek at the current byte without advancing
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    /// Advance by one byte
    #[inline]
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Skip whitespace characters efficiently using a lookup table
    #[inline]
    fn skip_whitespace(&mut self) {
        let input = self.input;
        let mut pos = self.pos;
        unsafe {
            while pos < self.len && *WS.get_unchecked(input[pos] as usize) {
                pos += 1;
            }
        }
        self.pos = pos;
    }

    /// Check if next non-whitespace character is a string quote
    #[inline]
    pub fn peek_is_string(&mut self) -> Result<bool> {
        self.skip_whitespace();
        Ok(self.peek() == Some(b'"'))
    }

    /// Parse a complete JSON value
    pub fn parse_value(&mut self) -> Result<JsonValue> {
        self.skip_whitespace();

        if self.depth > MAX_DEPTH {
            return Err(JsonError::NestingTooDeep(self.depth));
        }

        let byte = self.peek().ok_or(JsonError::UnexpectedEnd)?;

        match byte {
            b'"' => self
                .parse_string()
                .map(|s| JsonValue::String(s.into_owned())),
            b'{' => self.parse_object_value(),
            b'[' => self.parse_array_value(),
            b't' => self.parse_true().map(|_| JsonValue::Bool(true)),
            b'f' => self.parse_false().map(|_| JsonValue::Bool(false)),
            b'n' => self.parse_null().map(|_| JsonValue::Null),
            b'-' | b'0'..=b'9' => self.parse_number_value(),
            _ => Err(JsonError::UnexpectedChar(byte as char, self.pos)),
        }
    }

    /// Parse a string, returning a Cow to avoid allocation when possible
    pub fn parse_string(&mut self) -> Result<Cow<'a, str>> {
        self.skip_whitespace();

        if self.peek() != Some(b'"') {
            return Err(JsonError::ExpectedToken("string", self.pos));
        }
        self.advance();

        let start = self.pos;
        let mut has_escapes = false;

        unsafe {
            // Fast path: scan for end quote or escape
            while self.pos < self.len {
                match self.input.get_unchecked(self.pos) {
                    // bounds check once per loop
                    b'"' => {
                        if has_escapes {
                            // Need to process escapes
                            let raw = &self.input.get_unchecked(start..self.pos);
                            self.advance(); // consume closing quote
                            return self.unescape_string(raw);
                        } else {
                            // Zero-copy path: no escapes found
                            let s = std::str::from_utf8_unchecked(
                                self.input.get_unchecked(start..self.pos),
                            );
                            self.advance(); // consume closing quote
                            return Ok(Cow::Borrowed(s));
                        }
                    }
                    b'\\' => {
                        has_escapes = true;
                        self.pos += 2; // skip escape sequence
                    }
                    _ => self.pos += 1,
                }
            }
        }

        Err(JsonError::UnexpectedEnd)
    }

    /// Unescape a string with escape sequences
    fn unescape_string(&self, raw: &[u8]) -> Result<Cow<'a, str>> {
        let mut result = Vec::with_capacity(raw.len());
        let mut i = 0;

        while i < raw.len() {
            if raw[i] == b'\\' {
                i += 1;
                if i >= raw.len() {
                    return Err(JsonError::InvalidEscape(self.pos));
                }
                match raw[i] {
                    b'"' => result.push(b'"'),
                    b'\\' => result.push(b'\\'),
                    b'/' => result.push(b'/'),
                    b'b' => result.push(0x08),
                    b'f' => result.push(0x0C),
                    b'n' => result.push(b'\n'),
                    b'r' => result.push(b'\r'),
                    b't' => result.push(b'\t'),
                    b'u' => {
                        if i + 4 >= raw.len() {
                            return Err(JsonError::InvalidUnicode(self.pos));
                        }
                        let hex = &raw[i + 1..i + 5];
                        let code_point = self.parse_hex4(hex)?;

                        // Handle surrogate pairs
                        if (0xD800..=0xDBFF).contains(&code_point) {
                            // High surrogate, look for low surrogate
                            if i + 10 < raw.len() && raw[i + 5] == b'\\' && raw[i + 6] == b'u' {
                                let low_hex = &raw[i + 7..i + 11];
                                let low_code_point = self.parse_hex4(low_hex)?;
                                if (0xDC00..=0xDFFF).contains(&low_code_point) {
                                    let combined = 0x10000
                                        + ((code_point as u32 - 0xD800) << 10)
                                        + (low_code_point as u32 - 0xDC00);
                                    if let Some(c) = char::from_u32(combined) {
                                        let mut buf = [0u8; 4];
                                        result
                                            .extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                                    }
                                    // From position of 'u':
                                    //   4 hex of first code unit  = 4
                                    //   '\' + 'u' of second pair  = 2
                                    //   4 hex of second code unit = 4
                                    //   plus the final i+=1 below will NOT run (continue)
                                    //   so we must consume all 11 bytes ourselves
                                    i += 11;
                                    continue;
                                }
                            }
                        }

                        if let Some(c) = char::from_u32(code_point as u32) {
                            let mut buf = [0u8; 4];
                            result.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                        }
                        i += 4;
                    }
                    _ => return Err(JsonError::InvalidEscape(self.pos)),
                }
                i += 1;
            } else {
                result.push(raw[i]);
                i += 1;
            }
        }

        String::from_utf8(result)
            .map(Cow::Owned)
            .map_err(|_| JsonError::InvalidUtf8)
    }

    /// Parse 4 hex digits into a u16
    #[inline]
    fn parse_hex4(&self, hex: &[u8]) -> Result<u16> {
        let mut value = 0u16;
        for &b in hex {
            let digit = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => return Err(JsonError::InvalidUnicode(self.pos)),
            };
            value = value * 16 + digit as u16;
        }
        Ok(value)
    }

    /// Parse a number, returning as JsonValue
    fn parse_number_value(&mut self) -> Result<JsonValue> {
        let start = self.pos;
        let mut is_float = false;

        // Optional negative sign
        if self.peek() == Some(b'-') {
            self.advance();
        }

        // Integer part
        match self.peek() {
            Some(b'0') => self.advance(),
            Some(b'1'..=b'9') => {
                self.advance();
                while let Some(b'0'..=b'9') = self.peek() {
                    self.advance();
                }
            }
            _ => return Err(JsonError::InvalidNumber(start)),
        }

        // Fractional part
        if self.peek() == Some(b'.') {
            is_float = true;
            self.advance();
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError::InvalidNumber(self.pos));
            }
            while let Some(b'0'..=b'9') = self.peek() {
                self.advance();
            }
        }

        // Exponent part
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.advance();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.advance();
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError::InvalidNumber(self.pos));
            }
            while let Some(b'0'..=b'9') = self.peek() {
                self.advance();
            }
        }

        let num_str = unsafe { std::str::from_utf8_unchecked(&self.input[start..self.pos]) };

        if is_float {
            num_str
                .parse::<f64>()
                .map(JsonValue::Float)
                .map_err(|_| JsonError::InvalidNumber(start))
        } else {
            num_str
                .parse::<i64>()
                .map(JsonValue::Integer)
                .map_err(|_| JsonError::InvalidNumber(start))
        }
    }

    /// Parse a signed integer directly from bytes without going through str::parse.
    /// Handles i8/i16/i32/i64/isize and u8/u16/u32/u64/usize via the `ParseInt` helper trait.
    pub fn parse_integer<T: ParseInt>(&mut self) -> Result<T> {
        self.skip_whitespace();
        let start = self.pos;
        let negative = self.input.get(self.pos) == Some(&b'-');
        if negative {
            self.pos += 1;
        }

        if !matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
            return Err(JsonError::InvalidNumber(start));
        }

        let mut value: u64 = 0;
        while let Some(&d @ b'0'..=b'9') = self.input.get(self.pos) {
            value = value.wrapping_mul(10).wrapping_add((d - b'0') as u64);
            self.pos += 1;
        }

        T::from_parts(negative, value).ok_or(JsonError::InvalidNumber(start))
    }

    /// Parse a floating-point number
    pub fn parse_float<T: std::str::FromStr>(&mut self) -> Result<T> {
        self.skip_whitespace();
        let start = self.pos;

        if self.input.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }

        match self.input.get(self.pos) {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(JsonError::InvalidNumber(start)),
        }

        if self.input.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            while matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        if matches!(self.input.get(self.pos), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.input.get(self.pos), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        let num_str = unsafe { std::str::from_utf8_unchecked(&self.input[start..self.pos]) };
        num_str.parse().map_err(|_| JsonError::InvalidNumber(start))
    }

    /// Parse 'true'
    fn parse_true(&mut self) -> Result<()> {
        if self.input[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(())
        } else {
            Err(JsonError::ExpectedToken("true", self.pos))
        }
    }

    /// Parse 'false'
    fn parse_false(&mut self) -> Result<()> {
        if self.input[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(())
        } else {
            Err(JsonError::ExpectedToken("false", self.pos))
        }
    }

    /// Parse 'null'
    fn parse_null(&mut self) -> Result<()> {
        if self.input[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(())
        } else {
            Err(JsonError::ExpectedToken("null", self.pos))
        }
    }

    /// Parse a boolean value
    pub fn parse_bool(&mut self) -> Result<bool> {
        self.skip_whitespace();
        match self.peek() {
            Some(b't') => {
                self.parse_true()?;
                Ok(true)
            }
            Some(b'f') => {
                self.parse_false()?;
                Ok(false)
            }
            _ => Err(JsonError::ExpectedToken("boolean", self.pos)),
        }
    }

    /// Parse an object into JsonValue
    fn parse_object_value(&mut self) -> Result<JsonValue> {
        self.depth += 1;
        self.advance(); // consume '{'
        self.skip_whitespace();

        let mut map = Vec::new();

        if self.peek() == Some(b'}') {
            self.advance();
            self.depth -= 1;
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?.into_owned();

            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(JsonError::ExpectedChar(':', self.pos));
            }
            self.advance();

            let value = self.parse_value()?;
            map.push((key, value));

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.advance(),
                Some(b'}') => {
                    self.advance();
                    self.depth -= 1;
                    return Ok(JsonValue::Object(map));
                }
                _ => return Err(JsonError::ExpectedChar('}', self.pos)),
            }
        }
    }

    /// Parse an array into JsonValue
    fn parse_array_value(&mut self) -> Result<JsonValue> {
        self.depth += 1;
        self.advance(); // consume '['
        self.skip_whitespace();

        let mut arr = Vec::new();

        if self.peek() == Some(b']') {
            self.advance();
            self.depth -= 1;
            return Ok(JsonValue::Array(arr));
        }

        loop {
            arr.push(self.parse_value()?);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.advance(),
                Some(b']') => {
                    self.advance();
                    self.depth -= 1;
                    return Ok(JsonValue::Array(arr));
                }
                _ => return Err(JsonError::ExpectedChar(']', self.pos)),
            }
        }
    }

    /// Skip whitespace (public version for traits)
    #[inline]
    pub fn skip_whitespace_pub(&mut self) {
        self.skip_whitespace();
    }

    /// Check if next value is null (without consuming)
    #[inline]
    pub fn peek_is_null(&mut self) -> bool {
        self.skip_whitespace();
        self.input[self.pos..].starts_with(b"null")
    }

    /// Check for next array element, handling first element specially
    #[inline]
    pub fn has_next_array_element_or_first(&mut self, is_first: bool) -> Result<bool> {
        self.skip_whitespace();
        match self.peek() {
            Some(b']') => Ok(false),
            Some(b',') if !is_first => {
                self.advance();
                self.skip_whitespace();
                Ok(self.peek() != Some(b']'))
            }
            Some(_) if is_first => Ok(true),
            Some(c) => Err(JsonError::UnexpectedChar(c as char, self.pos)),
            None => Err(JsonError::UnexpectedEnd),
        }
    }

    // ========== Streaming API for derive macros ==========

    /// Expect and consume '{'
    pub fn expect_object_start(&mut self) -> Result<()> {
        self.skip_whitespace();
        if self.peek() != Some(b'{') {
            return Err(JsonError::ExpectedChar('{', self.pos));
        }
        self.advance();
        self.depth += 1;
        Ok(())
    }

    /// Expect and consume '}'
    pub fn expect_object_end(&mut self) -> Result<()> {
        self.skip_whitespace();
        if self.peek() != Some(b'}') {
            return Err(JsonError::ExpectedChar('}', self.pos));
        }
        self.advance();
        self.depth -= 1;
        Ok(())
    }

    /// Expect and consume '['
    pub fn expect_array_start(&mut self) -> Result<()> {
        self.skip_whitespace();
        if self.peek() != Some(b'[') {
            return Err(JsonError::ExpectedChar('[', self.pos));
        }
        self.advance();
        self.depth += 1;
        Ok(())
    }

    /// Expect and consume ']'
    pub fn expect_array_end(&mut self) -> Result<()> {
        self.skip_whitespace();
        if self.peek() != Some(b']') {
            return Err(JsonError::ExpectedChar(']', self.pos));
        }
        self.advance();
        self.depth -= 1;
        Ok(())
    }

    /// Expect and consume ','
    pub fn expect_comma(&mut self) -> Result<()> {
        self.skip_whitespace();
        if self.peek() != Some(b',') {
            return Err(JsonError::ExpectedChar(',', self.pos));
        }
        self.advance();
        Ok(())
    }

    /// Expect and consume 'null'
    pub fn expect_null(&mut self) -> Result<()> {
        self.skip_whitespace();
        self.parse_null()
    }

    /// Get the next object key, or None if at end of object
    pub fn next_object_key(&mut self) -> Result<Option<Cow<'a, str>>> {
        self.skip_whitespace();

        match self.peek() {
            Some(b'}') => Ok(None),
            Some(b',') => {
                self.advance();
                self.skip_whitespace();
                if self.peek() == Some(b'}') {
                    return Ok(None);
                }
                let key = self.parse_string()?;
                self.skip_whitespace();
                if self.peek() != Some(b':') {
                    return Err(JsonError::ExpectedChar(':', self.pos));
                }
                self.advance();
                Ok(Some(key))
            }
            Some(b'"') => {
                let key = self.parse_string()?;
                self.skip_whitespace();
                if self.peek() != Some(b':') {
                    return Err(JsonError::ExpectedChar(':', self.pos));
                }
                self.advance();
                Ok(Some(key))
            }
            Some(c) => Err(JsonError::UnexpectedChar(c as char, self.pos)),
            None => Err(JsonError::UnexpectedEnd),
        }
    }

    /// Check if there's another array element
    pub fn has_next_array_element(&mut self) -> Result<bool> {
        self.skip_whitespace();
        match self.peek() {
            Some(b']') => Ok(false),
            Some(b',') => {
                self.advance();
                self.skip_whitespace();
                Ok(self.peek() != Some(b']'))
            }
            Some(_) => Ok(true),
            None => Err(JsonError::UnexpectedEnd),
        }
    }

    /// Skip a value (for unknown fields) without allocating
    pub fn skip_value(&mut self) -> Result<()> {
        self.skip_whitespace();
        match self.input.get(self.pos) {
            Some(b'"') => unsafe {
                // Fast skip: scan for closing quote, skipping over escapes
                self.pos += 1; // consume opening quote
                while self.pos < self.len {
                    match self.input.get_unchecked(self.pos) {
                        b'"' => {
                            self.pos += 1;
                            return Ok(());
                        }
                        b'\\' => {
                            self.pos += 2;
                        } // skip escape pair
                        _ => {
                            self.pos += 1;
                        }
                    }
                }

                Err(JsonError::UnexpectedEnd)
            },
            Some(b'{') => unsafe {
                self.pos += 1;
                let mut depth = 1usize;
                while self.pos < self.len {
                    // bounds check once per loop
                    match self.input.get_unchecked(self.pos) {
                        b'"' => {
                            self.pos += 1;
                            self.skip_string_body()?;
                        }
                        b'{' | b'[' => {
                            depth += 1;
                            self.pos += 1;
                        }
                        b'}' | b']' => {
                            self.pos += 1;
                            depth -= 1;
                            if depth == 0 {
                                return Ok(());
                            }
                        }
                        _ => {
                            self.pos += 1;
                        }
                    }
                }
                Err(JsonError::UnexpectedEnd)
            },
            Some(b'[') => unsafe {
                self.pos += 1;
                let mut depth = 1usize;
                while self.pos < self.len {
                    match self.input.get_unchecked(self.pos) {
                        b'"' => {
                            self.pos += 1;
                            self.skip_string_body()?;
                        }
                        b'{' | b'[' => {
                            depth += 1;
                            self.pos += 1;
                        }
                        b'}' | b']' => {
                            self.pos += 1;
                            depth -= 1;
                            if depth == 0 {
                                return Ok(());
                            }
                        }
                        _ => {
                            self.pos += 1;
                        }
                    }
                }
                Err(JsonError::UnexpectedEnd)
            },
            Some(b't') => {
                self.pos += 4;
                Ok(())
            } // "true"
            Some(b'f') => {
                self.pos += 5;
                Ok(())
            } // "false"
            Some(b'n') => {
                self.pos += 4;
                Ok(())
            } // "null"
            Some(b'-') | Some(b'0'..=b'9') => {
                // Skip past all number characters
                if self.input.get(self.pos) == Some(&b'-') {
                    self.pos += 1;
                }
                while matches!(
                    self.input.get(self.pos),
                    Some(b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                ) {
                    self.pos += 1;
                }
                Ok(())
            }
            Some(&c) => Err(JsonError::UnexpectedChar(c as char, self.pos)),
            None => Err(JsonError::UnexpectedEnd),
        }
    }

    /// Skip over the body of a string (after the opening quote has been consumed)
    #[inline]
    fn skip_string_body(&mut self) -> Result<()> {
        loop {
            match self.input.get(self.pos) {
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(b'\\') => {
                    self.pos += 2;
                }
                Some(_) => {
                    self.pos += 1;
                }
                None => return Err(JsonError::UnexpectedEnd),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_simple() {
        let mut parser = JsonParser::new(r#""hello""#);
        assert_eq!(parser.parse_string().unwrap(), "hello");
    }

    #[test]
    fn test_parse_string_escapes() {
        let mut parser = JsonParser::new(r#""hello\nworld""#);
        assert_eq!(parser.parse_string().unwrap(), "hello\nworld");
    }

    #[test]
    fn test_parse_string_unicode() {
        let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#);
        assert_eq!(parser.parse_string().unwrap(), "Hello");
    }

    #[test]
    fn test_parse_number_integer() {
        let mut parser = JsonParser::new("42");
        match parser.parse_value().unwrap() {
            JsonValue::Integer(n) => assert_eq!(n, 42),
            _ => panic!("expected integer"),
        }
    }

    #[test]
    fn test_parse_number_negative() {
        let mut parser = JsonParser::new("-123");
        match parser.parse_value().unwrap() {
            JsonValue::Integer(n) => assert_eq!(n, -123),
            _ => panic!("expected integer"),
        }
    }

    #[test]
    fn test_parse_number_float() {
        let mut parser = JsonParser::new("3.14");
        match parser.parse_value().unwrap() {
            JsonValue::Float(n) => assert!((n - 3.14).abs() < 0.001),
            _ => panic!("expected float"),
        }
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert!(parser.parse_bool().unwrap());

        let mut parser = JsonParser::new("false");
        assert!(!parser.parse_bool().unwrap());
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let value = parser.parse_value().unwrap();
        assert!(value.is_array());
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let value = parser.parse_value().unwrap();
        assert!(value.is_object());
    }
}
