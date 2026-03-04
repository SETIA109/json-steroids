# Profiling Examples

This folder contains standalone binaries for profiling various aspects of json-steroids.

## Quick Start

```bash
# Compile all examples
cargo build --release --examples

# Run with samply
samply record target/release/examples/prof_serialize_complex
```

## Available Examples

### Serialization
- **prof_serialize_simple** - simple structure (3 fields)
- **prof_serialize_complex** - complex structure with nested objects
- **prof_serialize_large_array** - array of 1000 elements
- **prof_serialize_string_escapes** - strings with escape sequences

### Deserialization
- **prof_deserialize_simple** - simple structure
- **prof_deserialize_complex** - complex structure

### Combined
- **prof_roundtrip_complex** - serialization + deserialization
- **prof_parse_dynamic** - parsing to JsonValue
- **prof_realistic_api** - realistic API response

### JsonWriter API
- **prof_writer_manual** - manual JSON construction

## Performance

Typical results on Apple Silicon (M1/M2/M3):

```
prof_serialize_simple:      ~80ns  per iteration
prof_serialize_complex:     ~450ns per iteration
prof_deserialize_simple:    ~200ns per iteration
prof_deserialize_complex:   ~600ns per iteration
prof_roundtrip_complex:     ~1.1µs per iteration
```

## Modification

To change the number of iterations, edit the constant in the desired file:

```rust
const ITERS: usize = 100_000; // Change here
```

Then recompile:

```bash
cargo build --release --example prof_serialize_complex
```
