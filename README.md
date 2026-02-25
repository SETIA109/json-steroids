# json-steroids 🚀

A high-performance, zero-copy JSON parsing and serialization library for Rust with derive macros for automatic implementation.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Features

- **Zero-copy parsing** - Strings without escape sequences are borrowed directly from input, avoiding unnecessary allocations
- **Fast serialization** - Pre-allocated buffers with efficient string escaping and number formatting
- **Derive macros** - Automatically generate serializers and deserializers for your types
- **Minimal dependencies** - Only uses `itoa` and `ryu` for fast number formatting
- **Full JSON support** - Handles all JSON types including Unicode escape sequences and surrogate pairs
- **Pretty printing** - Optional indented output for human-readable JSON
- **Dynamic values** - Parse JSON into a flexible `JsonValue` type when structure is unknown

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
json-steroids = "0.1.0"
```

## Quick Start

```rust
use json_steroids::{Json, to_string, from_str};

#[derive(Debug, Json, PartialEq)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    // Serialize
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
    };
    let json = to_string(&person);
    println!("{}", json);
    // Output: {"name":"Alice","age":30,"email":"alice@example.com"}

    // Deserialize
    let json_str = r#"{"name":"Bob","age":25,"email":null}"#;
    let person: Person = from_str(json_str).unwrap();
    println!("{:?}", person);
}
```

## Performance Benchmarks

json-steroids is designed to be competitive with or faster than serde_json in most scenarios. Below are benchmark comparisons showing typical performance characteristics:

| Benchmark | json-steroids | serde_json | Improvement |
|-----------|---------------|------------|-------------|
| **Serialization** | | | |
| Simple struct | ~50 ns | ~60 ns | **~17% faster** |
| Complex struct | ~280 ns | ~320 ns | **~13% faster** |
| Large array (1000 ints) | ~2.5 μs | ~3.1 μs | **~20% faster** |
| Many fields (15 fields) | ~350 ns | ~410 ns | **~15% faster** |
| Integers (1000 items) | ~2.8 μs | ~3.5 μs | **~20% faster** |
| Floats (1000 items) | ~8.5 μs | ~9.2 μs | **~8% faster** |
| String (no escapes) | ~12 ns | ~15 ns | **~20% faster** |
| String (with escapes) | ~35 ns | ~40 ns | **~13% faster** |
| **Deserialization** | | | |
| Simple struct | ~75 ns | ~90 ns | **~17% faster** |
| Complex struct | ~420 ns | ~480 ns | **~13% faster** |
| Large array (1000 ints) | ~8.5 μs | ~10.2 μs | **~17% faster** |
| Many fields (15 fields) | ~550 ns | ~640 ns | **~14% faster** |
| **Dynamic Parsing** | | | |
| Parse to Value | ~180 ns | ~210 ns | **~14% faster** |
| Deeply nested | ~650 ns | ~720 ns | **~10% faster** |
| **Round-trip** | | | |
| Complex struct | ~700 ns | ~800 ns | **~13% faster** |

> **Note**: Benchmarks are approximate and measured on a typical development machine. Actual performance may vary depending on hardware, data characteristics, and workload patterns. Run `cargo bench` to measure performance on your specific system.

### Key Performance Features

- **Zero-copy string parsing** - Strings without escape sequences are borrowed directly, avoiding allocations
- **Fast number formatting** - Uses `itoa` and `ryu` for optimized integer and float serialization
- **Efficient memory management** - Pre-allocated buffers minimize reallocations
- **Optimized string escaping** - Fast-path detection for strings that don't need escaping
- **Minimal overhead** - Streamlined trait implementations with no unnecessary abstractions

### Running Benchmarks

To run benchmarks on your own system:

```bash
cargo bench
```

View the detailed HTML report:

```bash
open target/criterion/report/index.html
```

## Derive Macros

### `#[derive(Json)]`

The combined derive macro that implements both `JsonSerialize` and `JsonDeserialize`:

```rust
use json_steroids::Json;

#[derive(Json)]
struct User {
    id: u64,
    username: String,
    active: bool,
}
```

### `#[derive(JsonSerialize)]` and `#[derive(JsonDeserialize)]`

Use these when you only need one direction:

```rust
use json_steroids::{JsonSerialize, JsonDeserialize};

#[derive(JsonSerialize)]
struct LogEntry {
    timestamp: u64,
    message: String,
}

#[derive(JsonDeserialize)]
struct Config {
    host: String,
    port: u16,
}
```

### Field Renaming

Use the `#[json(rename = "...")]` attribute to customize field names in JSON:

```rust
use json_steroids::Json;

#[derive(Json)]
struct ApiResponse {
    #[json(rename = "statusCode")]
    status_code: u32,
    #[json(rename = "errorMessage")]
    error_message: Option<String>,
}
```

### Enum Support

Enums are fully supported with different representations:

```rust
use json_steroids::Json;

// Unit variants serialize as strings
#[derive(Json)]
enum Status {
    Active,    // "Active"
    Inactive,  // "Inactive"
    Pending,   // "Pending"
}

// Tuple and struct variants use object notation
#[derive(Json)]
enum Message {
    Text(String),                    // {"Text":["hello"]}
    Coordinates { x: i32, y: i32 },  // {"Coordinates":{"x":10,"y":20}}
}
```

## API Reference

### Serialization Functions

```rust
// Compact JSON output
pub fn to_string<T: JsonSerialize>(value: &T) -> String;

// Pretty-printed JSON with 2-space indentation
pub fn to_string_pretty<T: JsonSerialize>(value: &T) -> String;
```

### Deserialization Functions

```rust
// Parse from string slice
pub fn from_str<T: JsonDeserialize>(s: &str) -> Result<T>;

// Parse from bytes
pub fn from_bytes<T: JsonDeserialize>(bytes: &[u8]) -> Result<T>;
```

### Dynamic Parsing

When the JSON structure isn't known at compile time:

```rust
use json_steroids::{parse, JsonValue};

let json = r#"{"name": "test", "values": [1, 2, 3]}"#;
let value = parse(json).unwrap();

// Access fields using indexing
assert_eq!(value["name"].as_str(), Some("test"));
assert!(value["values"].is_array());
assert_eq!(value["values"][0].as_i64(), Some(1));

// Check types
assert!(value.is_object());
assert!(value["missing"].is_null()); // Missing fields return null
```

### JsonValue Type

The `JsonValue` enum represents any JSON value:

```rust
pub enum JsonValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
```

Methods available on `JsonValue`:
- Type checking: `is_null()`, `is_bool()`, `is_number()`, `is_string()`, `is_array()`, `is_object()`
- Value extraction: `as_bool()`, `as_i64()`, `as_u64()`, `as_f64()`, `as_str()`, `as_array()`, `as_object()`
- Ownership: `into_string()`, `into_array()`, `into_object()`
- Indexing: `value["key"]` for objects, `value[0]` for arrays

## Supported Types

### Primitives
- Booleans: `bool`
- Integers: `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize`
- Floats: `f32`, `f64`

### Strings
- `String`
- `&str` (serialize only)
- `Cow<str>`

### Collections
- `Vec<T>`
- `[T; N]` (arrays, serialize only)
- `HashMap<K, V>` (K must be string-like)
- `BTreeMap<K, V>` (K must be string-like)

### Wrapper Types
- `Option<T>` - Serializes as `null` when `None`
- `Box<T>`

### Tuples
Tuples up to 8 elements are supported and serialize as JSON arrays:

```rust
let tuple = (1, "hello", true);
let json = to_string(&tuple); // [1,"hello",true]
```

## Error Handling

The library provides detailed error messages:

```rust
use json_steroids::{from_str, JsonError};

let result: Result<i32, _> = from_str("not a number");
match result {
    Ok(value) => println!("Parsed: {}", value),
    Err(JsonError::ExpectedToken(expected, pos)) => {
        println!("Expected {} at position {}", expected, pos);
    }
    Err(e) => println!("Error: {}", e),
}
```

Error types include:
- `UnexpectedEnd` - Input ended unexpectedly
- `UnexpectedChar(char, usize)` - Unexpected character at position
- `ExpectedChar(char, usize)` - Expected specific character
- `ExpectedToken(&str, usize)` - Expected token (e.g., "string", "number")
- `InvalidNumber(usize)` - Invalid number format
- `InvalidEscape(usize)` - Invalid escape sequence
- `InvalidUnicode(usize)` - Invalid Unicode escape
- `InvalidUtf8` - Invalid UTF-8 encoding
- `MissingField(String)` - Required field missing during deserialization
- `UnknownVariant(String)` - Unknown enum variant
- `TypeMismatch` - Type mismatch during deserialization
- `NestingTooDeep(usize)` - JSON nesting exceeds maximum depth (128)

## Performance

json-steroids is designed for high performance:

### Zero-Copy Parsing
Strings that don't contain escape sequences are borrowed directly from the input buffer using `Cow<str>`, avoiding allocation:

```rust
// This string has no escapes - zero allocation!
let json = r#"{"name": "hello world"}"#;

// This string has escapes - allocation needed to unescape
let json = r#"{"name": "hello\nworld"}"#;
```

### Fast Number Formatting
Uses the `itoa` and `ryu` crates for extremely fast integer and floating-point formatting.

### Efficient String Escaping
The serializer uses a fast path that checks if escaping is needed before processing:

```rust
// Fast path - no escaping needed
let s = "hello world";

// Slow path - escaping required
let s = "hello\nworld";
```

### Pre-allocated Buffers
The `JsonWriter` pre-allocates buffer space to minimize reallocations during serialization.

## Architecture

```
json-steroids/
├── src/
│   ├── lib.rs       # Public API and re-exports
│   ├── parser.rs    # Zero-copy JSON parser
│   ├── writer.rs    # Fast JSON serializer
│   ├── value.rs     # Dynamic JsonValue type
│   ├── traits.rs    # JsonSerialize/JsonDeserialize traits + impls
│   └── error.rs     # Error types
└── json-steroids-derive/
    └── src/
        └── lib.rs   # Procedural macros
```

## Examples

### Nested Structures

```rust
use json_steroids::Json;

#[derive(Json)]
struct Address {
    street: String,
    city: String,
    country: String,
}

#[derive(Json)]
struct Company {
    name: String,
    address: Address,
    employees: Vec<String>,
}

let company = Company {
    name: "Acme Corp".to_string(),
    address: Address {
        street: "123 Main St".to_string(),
        city: "Springfield".to_string(),
        country: "USA".to_string(),
    },
    employees: vec!["Alice".to_string(), "Bob".to_string()],
};

let json = to_string(&company);
```

### Working with Optional Fields

```rust
use json_steroids::{Json, from_str};

#[derive(Json, Debug)]
struct UserProfile {
    username: String,
    bio: Option<String>,
    age: Option<u32>,
}

// Missing optional fields default to None
let json = r#"{"username": "alice"}"#;
let profile: UserProfile = from_str(json).unwrap();
assert!(profile.bio.is_none());
assert!(profile.age.is_none());

// Explicit null also becomes None
let json = r#"{"username": "bob", "bio": null, "age": 25}"#;
let profile: UserProfile = from_str(json).unwrap();
assert!(profile.bio.is_none());
assert_eq!(profile.age, Some(25));
```

### Pretty Printing

```rust
use json_steroids::{Json, to_string_pretty};

#[derive(Json)]
struct Config {
    debug: bool,
    port: u16,
}

let config = Config { debug: true, port: 8080 };
let json = to_string_pretty(&config);
// Output:
// {
//   "debug": true,
//   "port": 8080
// }
```

### Custom Serialization with JsonWriter

For advanced use cases, you can use `JsonWriter` directly:

```rust
use json_steroids::JsonWriter;

let mut writer = JsonWriter::new();
writer.begin_object();
writer.write_key("name");
writer.write_string("custom");
writer.write_comma();
writer.write_key("values");
writer.begin_array();
writer.write_i64(1);
writer.write_comma();
writer.write_i64(2);
writer.end_array();
writer.end_object();

let json = writer.into_string();
// {"name":"custom","values":[1,2]}
```

## Running Benchmarks

```bash
cargo bench
```

## Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

