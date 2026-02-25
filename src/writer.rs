//! High-performance JSON writer
//!
//! Optimized for fast serialization with:
//! - Pre-allocated buffer
//! - Efficient string escaping
//! - Optional pretty printing

/// Lookup table: 0 = safe byte, 1 = needs escaping
/// Covers bytes 0x00-0xFF. Bytes < 0x20, '"', and '\\' need escaping.
static NEEDS_ESCAPE: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0u8;
    // Control characters 0x00-0x1F
    loop {
        table[i as usize] = true;
        if i == 0x1F { break; }
        i += 1;
    }
    table[b'"' as usize] = true;
    table[b'\\' as usize] = true;
    table
};

/// JSON writer with optimized string building
pub struct JsonWriter {
    /// Output buffer
    buffer: Vec<u8>,
    /// Indentation string (empty for compact)
    indent: Option<Vec<u8>>,
    /// Current indentation level
    depth: usize,
    /// Whether we need a newline before next content
    needs_newline: bool,
}

impl JsonWriter {
    /// Create a new compact JSON writer
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
            indent: None,
            depth: 0,
            needs_newline: false,
        }
    }

    /// Create a new writer with the given indentation
    #[inline]
    pub fn with_indent(spaces: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
            indent: Some(vec![b' '; spaces]),
            depth: 0,
            needs_newline: false,
        }
    }

    /// Create a writer with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            indent: None,
            depth: 0,
            needs_newline: false,
        }
    }

    /// Get the result as a String
    #[inline]
    pub fn into_string(self) -> String {
        // Safety: we only write valid UTF-8
        unsafe { String::from_utf8_unchecked(self.buffer) }
    }

    /// Get the result as bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Write indentation if pretty printing
    #[inline]
    fn write_indent(&mut self) {
        if let Some(ref indent) = self.indent {
            if self.needs_newline {
                self.buffer.push(b'\n');
                for _ in 0..self.depth {
                    self.buffer.extend_from_slice(indent);
                }
                self.needs_newline = false;
            }
        }
    }

    /// Begin an object
    #[inline]
    pub fn begin_object(&mut self) {
        self.write_indent();
        self.buffer.push(b'{');
        self.depth += 1;
        if self.indent.is_some() {
            self.needs_newline = true;
        }
    }

    /// End an object
    #[inline]
    pub fn end_object(&mut self) {
        self.depth -= 1;
        if self.indent.is_some() {
            self.needs_newline = true;
            self.write_indent();
        }
        self.buffer.push(b'}');
    }

    /// Begin an array
    #[inline]
    pub fn begin_array(&mut self) {
        self.write_indent();
        self.buffer.push(b'[');
        self.depth += 1;
        if self.indent.is_some() {
            self.needs_newline = true;
        }
    }

    /// End an array
    #[inline]
    pub fn end_array(&mut self) {
        self.depth -= 1;
        if self.indent.is_some() {
            self.needs_newline = true;
            self.write_indent();
        }
        self.buffer.push(b']');
    }

    /// Write a comma separator
    #[inline]
    pub fn write_comma(&mut self) {
        self.buffer.push(b',');
        if self.indent.is_some() {
            self.needs_newline = true;
        }
    }

    /// Write an object key
    #[inline]
    pub fn write_key(&mut self, key: &str) {
        self.write_indent();
        self.write_string(key);
        self.buffer.push(b':');
        if self.indent.is_some() {
            self.buffer.push(b' ');
        }
    }

    /// Write a string value with proper escaping (single-pass)
    #[inline]
    pub fn write_string(&mut self, s: &str) {
        self.write_indent();
        self.buffer.push(b'"');
        self.write_escaped_string(s);
        self.buffer.push(b'"');
    }

    /// Single-pass string escaping: copies clean byte runs in bulk,
    /// only pays the per-byte cost when an escape is actually needed.
    #[inline]
    fn write_escaped_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let mut start = 0; // start of the current clean run

        for (i, &byte) in bytes.iter().enumerate() {
            if NEEDS_ESCAPE[byte as usize] {
                // Flush the clean run up to this point
                if start < i {
                    self.buffer.extend_from_slice(&bytes[start..i]);
                }
                match byte {
                    b'"'  => self.buffer.extend_from_slice(b"\\\""),
                    b'\\' => self.buffer.extend_from_slice(b"\\\\"),
                    b'\n' => self.buffer.extend_from_slice(b"\\n"),
                    b'\r' => self.buffer.extend_from_slice(b"\\r"),
                    b'\t' => self.buffer.extend_from_slice(b"\\t"),
                    0x08  => self.buffer.extend_from_slice(b"\\b"),
                    0x0C  => self.buffer.extend_from_slice(b"\\f"),
                    _ => {
                        // Other control characters as \u00XX
                        self.buffer.extend_from_slice(b"\\u00");
                        self.buffer.push(HEX_CHARS[(byte >> 4) as usize]);
                        self.buffer.push(HEX_CHARS[(byte & 0x0F) as usize]);
                    }
                }
                start = i + 1;
            }
        }

        // Flush the remaining clean run
        if start < bytes.len() {
            self.buffer.extend_from_slice(&bytes[start..]);
        }
    }

    /// Write a raw string (no escaping, no quotes)
    #[inline]
    pub fn write_raw(&mut self, s: &str) {
        self.write_indent();
        self.buffer.extend_from_slice(s.as_bytes());
    }

    /// Write null
    #[inline]
    pub fn write_null(&mut self) {
        self.write_indent();
        self.buffer.extend_from_slice(b"null");
    }

    /// Write a boolean
    #[inline]
    pub fn write_bool(&mut self, value: bool) {
        self.write_indent();
        if value {
            self.buffer.extend_from_slice(b"true");
        } else {
            self.buffer.extend_from_slice(b"false");
        }
    }

    /// Write an integer
    #[inline]
    pub fn write_i64(&mut self, value: i64) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer.extend_from_slice(buffer.format(value).as_bytes());
    }

    /// Write an unsigned integer
    #[inline]
    pub fn write_u64(&mut self, value: u64) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer.extend_from_slice(buffer.format(value).as_bytes());
    }

    /// Write a float
    #[inline]
    pub fn write_f64(&mut self, value: f64) {
        self.write_indent();
        let mut buffer = ryu::Buffer::new();
        self.buffer.extend_from_slice(buffer.format(value).as_bytes());
    }
}

impl Default for JsonWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Hex character lookup table
const HEX_CHARS: [u8; 16] = *b"0123456789abcdef";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_string() {
        let mut writer = JsonWriter::new();
        writer.write_string("hello");
        assert_eq!(writer.into_string(), r#""hello""#);
    }

    #[test]
    fn test_write_escaped_string() {
        let mut writer = JsonWriter::new();
        writer.write_string("hello\nworld");
        assert_eq!(writer.into_string(), r#""hello\nworld""#);
    }

    #[test]
    fn test_write_object() {
        let mut writer = JsonWriter::new();
        writer.begin_object();
        writer.write_key("name");
        writer.write_string("test");
        writer.end_object();
        assert_eq!(writer.into_string(), r#"{"name":"test"}"#);
    }

    #[test]
    fn test_write_array() {
        let mut writer = JsonWriter::new();
        writer.begin_array();
        writer.write_i64(1);
        writer.write_comma();
        writer.write_i64(2);
        writer.write_comma();
        writer.write_i64(3);
        writer.end_array();
        assert_eq!(writer.into_string(), "[1,2,3]");
    }

    #[test]
    fn test_pretty_print() {
        let mut writer = JsonWriter::with_indent(2);
        writer.begin_object();
        writer.write_key("name");
        writer.write_string("test");
        writer.end_object();
        let result = writer.into_string();
        assert!(result.contains('\n'));
    }
}
