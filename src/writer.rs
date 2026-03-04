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
        if i == 0x1F {
            break;
        }
        i += 1;
    }
    table[b'"' as usize] = true;
    table[b'\\' as usize] = true;
    table
};

/// Internal writer trait for different formatting strategies
pub trait Writer {
    fn buffer(&self) -> &Vec<u8>;
    fn buffer_mut(&mut self) -> &mut Vec<u8>;
    fn into_buffer(self) -> Vec<u8>;

    fn begin_object(&mut self);
    fn end_object(&mut self);
    fn begin_array(&mut self);
    fn end_array(&mut self);
    fn write_comma(&mut self);
    fn write_key(&mut self, key: &str);
    fn write_unescape_key(&mut self, s: &str);
    fn write_string(&mut self, s: &str);
    fn write_raw(&mut self, s: &str);
    fn write_null(&mut self);
    fn write_bool(&mut self, value: bool);
    fn write_i8(&mut self, value: i8);
    fn write_i16(&mut self, value: i16);
    fn write_i32(&mut self, value: i32);
    fn write_i64(&mut self, value: i64);
    fn write_isize(&mut self, value: isize);
    fn write_u8(&mut self, value: u8);
    fn write_u16(&mut self, value: u16);
    fn write_u32(&mut self, value: u32);
    fn write_u64(&mut self, value: u64);
    fn write_usize(&mut self, value: usize);
    fn write_f32(&mut self, value: f32);
    fn write_f64(&mut self, value: f64);
}

/// Compact JSON writer (no indentation)
pub struct CompactWriter {
    buffer: Vec<u8>,
    itoa_buffer: itoa::Buffer,
    ryu_buffer: ryu::Buffer,
}

/// Pretty-printed JSON writer (with indentation)
pub struct PrettyWriter {
    buffer: Vec<u8>,
    indent: Vec<u8>,
    depth: usize,
    needs_newline: bool,
}

/// JSON writer with optimized string building
pub struct JsonWriter<W: Writer = CompactWriter> {
    inner: W,
}

impl CompactWriter {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            itoa_buffer: itoa::Buffer::new(),
            ryu_buffer: ryu::Buffer::new(),
        }
    }
}

impl Writer for CompactWriter {
    #[inline]
    fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    #[inline]
    fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }

    #[inline]
    fn begin_object(&mut self) {
        self.buffer.push(b'{');
    }

    #[inline]
    fn end_object(&mut self) {
        self.buffer.push(b'}');
    }

    #[inline]
    fn begin_array(&mut self) {
        self.buffer.push(b'[');
    }

    #[inline]
    fn end_array(&mut self) {
        self.buffer.push(b']');
    }

    #[inline]
    fn write_comma(&mut self) {
        self.buffer.push(b',');
    }

    #[inline]
    fn write_key(&mut self, key: &str) {
        self.buffer.push(b'"');
        write_escaped_string(&mut self.buffer, key);
        self.buffer.extend_from_slice(b"\":");
    }

    #[inline]
    fn write_unescape_key(&mut self, s: &str) {
        self.buffer.push(b'"');
        self.buffer.extend_from_slice(s.as_bytes());
        self.buffer.extend_from_slice(b"\":");
    }

    #[inline]
    fn write_string(&mut self, s: &str) {
        self.buffer.push(b'"');
        write_escaped_string(&mut self.buffer, s);
        self.buffer.push(b'"');
    }

    #[inline]
    fn write_raw(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
    }

    #[inline]
    fn write_null(&mut self) {
        self.buffer.extend_from_slice(b"null");
    }

    #[inline]
    fn write_bool(&mut self, value: bool) {
        if value {
            self.buffer.extend_from_slice(b"true");
        } else {
            self.buffer.extend_from_slice(b"false");
        }
    }

    #[inline]
    fn write_i8(&mut self, value: i8) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_i16(&mut self, value: i16) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_i32(&mut self, value: i32) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_i64(&mut self, value: i64) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_isize(&mut self, value: isize) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.buffer
            .extend_from_slice(self.itoa_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_f32(&mut self, value: f32) {
        self.buffer
            .extend_from_slice(self.ryu_buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_f64(&mut self, value: f64) {
        self.buffer
            .extend_from_slice(self.ryu_buffer.format(value).as_bytes());
    }
}

impl PrettyWriter {
    #[inline]
    pub fn new(capacity: usize, spaces: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            indent: vec![b' '; spaces],
            depth: 0,
            needs_newline: false,
        }
    }

    #[inline]
    fn write_indent(&mut self) {
        if self.needs_newline {
            self.buffer.push(b'\n');
            for _ in 0..self.depth {
                self.buffer.extend_from_slice(&self.indent);
            }
            self.needs_newline = false;
        }
    }
}

impl Writer for PrettyWriter {
    #[inline]
    fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    #[inline]
    fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }

    #[inline]
    fn begin_object(&mut self) {
        self.write_indent();
        self.buffer.push(b'{');
        self.depth += 1;
        self.needs_newline = true;
    }

    #[inline]
    fn end_object(&mut self) {
        self.depth -= 1;
        self.needs_newline = true;
        self.write_indent();
        self.buffer.push(b'}');
    }

    #[inline]
    fn begin_array(&mut self) {
        self.write_indent();
        self.buffer.push(b'[');
        self.depth += 1;
        self.needs_newline = true;
    }

    #[inline]
    fn end_array(&mut self) {
        self.depth -= 1;
        self.needs_newline = true;
        self.write_indent();
        self.buffer.push(b']');
    }

    #[inline]
    fn write_comma(&mut self) {
        self.buffer.push(b',');
        self.needs_newline = true;
    }

    #[inline]
    fn write_key(&mut self, key: &str) {
        self.write_indent();
        self.buffer.push(b'"');
        write_escaped_string(&mut self.buffer, key);
        self.buffer.extend_from_slice(b"\": ");
    }

    #[inline]
    fn write_unescape_key(&mut self, key: &str) {
        self.write_indent();
        self.buffer.push(b'"');
        self.buffer.extend_from_slice(key.as_bytes());
        self.buffer.extend_from_slice(b"\": ");
    }

    #[inline]
    fn write_string(&mut self, s: &str) {
        self.write_indent();
        self.buffer.push(b'"');
        write_escaped_string(&mut self.buffer, s);
        self.buffer.push(b'"');
    }

    #[inline]
    fn write_raw(&mut self, s: &str) {
        self.write_indent();
        self.buffer.extend_from_slice(s.as_bytes());
    }

    #[inline]
    fn write_null(&mut self) {
        self.write_indent();
        self.buffer.extend_from_slice(b"null");
    }

    #[inline]
    fn write_bool(&mut self, value: bool) {
        self.write_indent();
        if value {
            self.buffer.extend_from_slice(b"true");
        } else {
            self.buffer.extend_from_slice(b"false");
        }
    }

    #[inline]
    fn write_i8(&mut self, value: i8) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_i16(&mut self, value: i16) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_i32(&mut self, value: i32) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_i64(&mut self, value: i64) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_isize(&mut self, value: isize) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.write_indent();
        let mut buffer = itoa::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_f32(&mut self, value: f32) {
        self.write_indent();
        let mut buffer = ryu::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    fn write_f64(&mut self, value: f64) {
        self.write_indent();
        let mut buffer = ryu::Buffer::new();
        self.buffer
            .extend_from_slice(buffer.format(value).as_bytes());
    }
}

/// Single-pass string escaping: copies clean byte runs in bulk,
/// only pays the per-byte cost when an escape is actually needed.
#[inline]
fn write_escaped_string(buffer: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0;

    for i in 0..bytes.len() {
        let byte = unsafe { *bytes.get_unchecked(i) };

        if unsafe { *NEEDS_ESCAPE.get_unchecked(byte as usize) } {
            // Write any accumulated clean bytes
            if start < i {
                buffer.extend_from_slice(&bytes[start..i]);
            }

            // Write the escape sequence
            buffer.push(b'\\');
            let escaped = match byte {
                b'"' => b'"',
                b'\\' => b'\\',
                b'\n' => b'n',
                b'\r' => b'r',
                b'\t' => b't',
                b'\x08' => b'b', // backspace
                b'\x0C' => b'f', // form feed
                _ => {
                    // Unicode escape for other control characters
                    buffer.push(b'u');
                    buffer.push(b'0');
                    buffer.push(b'0');
                    let hex_digits = b"0123456789abcdef";
                    buffer.push(hex_digits[(byte >> 4) as usize]);
                    buffer.push(hex_digits[(byte & 0x0F) as usize]);
                    start = i + 1;
                    continue;
                }
            };
            buffer.push(escaped);
            start = i + 1;
        }
    }

    // Write any remaining clean bytes
    if start < bytes.len() {
        buffer.extend_from_slice(&bytes[start..]);
    }
}

impl JsonWriter<CompactWriter> {
    /// Create a new compact JSON writer
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: CompactWriter::new(2048),
        }
    }

    /// Create a writer with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: CompactWriter::new(capacity),
        }
    }
}

impl JsonWriter<PrettyWriter> {
    /// Create a new writer with the given indentation
    #[inline]
    pub fn with_indent(spaces: usize) -> Self {
        Self {
            inner: PrettyWriter::new(1024, spaces),
        }
    }
}

impl<W: Writer> JsonWriter<W> {
    /// Get the result as a String
    #[inline]
    pub fn into_string(self) -> String {
        let buffer = self.inner.into_buffer();
        // Safety: we only write valid UTF-8
        unsafe { String::from_utf8_unchecked(buffer) }
    }

    /// Get the result as bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.buffer()
    }

    /// Begin an object
    #[inline]
    pub fn begin_object(&mut self) {
        self.inner.begin_object();
    }

    /// End an object
    #[inline]
    pub fn end_object(&mut self) {
        self.inner.end_object();
    }

    /// Begin an array
    #[inline]
    pub fn begin_array(&mut self) {
        self.inner.begin_array();
    }

    /// End an array
    #[inline]
    pub fn end_array(&mut self) {
        self.inner.end_array();
    }

    /// Write a comma separator
    #[inline]
    pub fn write_comma(&mut self) {
        self.inner.write_comma();
    }

    /// Write an object key
    #[inline]
    pub fn write_key(&mut self, key: &str) {
        self.inner.write_key(key);
    }

    /// Write an object key
    #[inline]
    pub fn write_unescape_key(&mut self, key: &str) {
        self.inner.write_unescape_key(key);
    }

    /// Write a string value with proper escaping (single-pass)
    #[inline]
    pub fn write_string(&mut self, s: &str) {
        self.inner.write_string(s);
    }

    /// Write a raw string (no escaping, no quotes)
    #[inline]
    pub fn write_raw(&mut self, s: &str) {
        self.inner.write_raw(s);
    }

    /// Write null
    #[inline]
    pub fn write_null(&mut self) {
        self.inner.write_null();
    }

    /// Write a boolean
    #[inline]
    pub fn write_bool(&mut self, value: bool) {
        self.inner.write_bool(value);
    }

    /// Write an i8
    #[inline]
    pub fn write_i8(&mut self, value: i8) {
        self.inner.write_i8(value);
    }

    /// Write an i16
    #[inline]
    pub fn write_i16(&mut self, value: i16) {
        self.inner.write_i16(value);
    }

    /// Write an i32
    #[inline]
    pub fn write_i32(&mut self, value: i32) {
        self.inner.write_i32(value);
    }

    /// Write an i64
    #[inline]
    pub fn write_i64(&mut self, value: i64) {
        self.inner.write_i64(value);
    }

    /// Write an isize
    #[inline]
    pub fn write_isize(&mut self, value: isize) {
        self.inner.write_isize(value);
    }

    /// Write a u8
    #[inline]
    pub fn write_u8(&mut self, value: u8) {
        self.inner.write_u8(value);
    }

    /// Write a u16
    #[inline]
    pub fn write_u16(&mut self, value: u16) {
        self.inner.write_u16(value);
    }

    /// Write a u32
    #[inline]
    pub fn write_u32(&mut self, value: u32) {
        self.inner.write_u32(value);
    }

    /// Write a u64
    #[inline]
    pub fn write_u64(&mut self, value: u64) {
        self.inner.write_u64(value);
    }

    /// Write a usize
    #[inline]
    pub fn write_usize(&mut self, value: usize) {
        self.inner.write_usize(value);
    }

    /// Write an f32
    #[inline]
    pub fn write_f32(&mut self, value: f32) {
        self.inner.write_f32(value);
    }

    /// Write an f64
    #[inline]
    pub fn write_f64(&mut self, value: f64) {
        self.inner.write_f64(value);
    }
}

impl Default for JsonWriter<CompactWriter> {
    fn default() -> Self {
        Self::new()
    }
}

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
