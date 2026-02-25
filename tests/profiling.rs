//! Profiling tests for json-steroids
//!
//! These tests are designed to be run under a profiler (e.g. `perf`, `flamegraph`,
//! `cargo-flamegraph`, `valgrind --tool=callgrind`).  Each test executes a hot
//! loop with a fixed, large iteration count so the profiler can collect a
//! statistically meaningful call-graph without needing criterion overhead.
//!
//! Run a single profile target, e.g.:
//!   cargo test --test profiling --release prof_serialize_simple -- --nocapture
//!
//! With flamegraph:
//!   cargo flamegraph --test profiling --test-name prof_serialize_complex -- --nocapture

use json_steroids::{
    from_str, from_bytes, parse, to_string, to_string_pretty,
    Json, JsonDeserialize, JsonSerialize, JsonValue, JsonWriter,
};
use std::collections::BTreeMap;
use std::hint::black_box;

// ─────────────────────────────────────────────────────────────────────────────
// Shared iteration count
// ─────────────────────────────────────────────────────────────────────────────

/// Number of iterations every profiling loop performs.
/// High enough to dominate profiler sampling; adjust if machine is too slow.
const ITERS: usize = 100_000;

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Json)]
struct SimpleStruct {
    name: String,
    value: i64,
    active: bool,
}

#[derive(Debug, PartialEq, Json)]
struct Metadata {
    created: String,
    updated: String,
    version: u32,
}

#[derive(Debug, PartialEq, Json)]
struct ComplexStruct {
    id: u64,
    name: String,
    score: f64,
    tags: Vec<String>,
    metadata: Metadata,
    values: Vec<i32>,
    maybe: Option<String>,
}

#[derive(Debug, PartialEq, Json)]
struct ManyFields {
    f1: String,
    f2: String,
    f3: String,
    f4: String,
    f5: String,
    n1: i32,
    n2: i32,
    n3: i32,
    n4: i32,
    n5: i32,
    b1: bool,
    b2: bool,
    d1: f64,
    d2: f64,
    opt: Option<String>,
}

#[derive(Debug, PartialEq, Json)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Debug, PartialEq, Json)]
enum Event {
    Click { x: f64, y: f64 },
    Key(String),
    Resize(u32, u32),
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture builders
// ─────────────────────────────────────────────────────────────────────────────

fn make_simple() -> SimpleStruct {
    SimpleStruct { name: "json-steroids".to_string(), value: 42, active: true }
}

fn make_complex() -> ComplexStruct {
    ComplexStruct {
        id: 9_999_999,
        name: "Profiling Subject".to_string(),
        score: 98.6,
        tags: vec![
            "rust".to_string(),
            "json".to_string(),
            "fast".to_string(),
            "zero-copy".to_string(),
        ],
        metadata: Metadata {
            created: "2026-01-01T00:00:00Z".to_string(),
            updated: "2026-02-25T12:00:00Z".to_string(),
            version: 7,
        },
        values: (0..20).collect(),
        maybe: Some("optional payload".to_string()),
    }
}

fn make_many_fields() -> ManyFields {
    ManyFields {
        f1: "alpha".to_string(),
        f2: "beta".to_string(),
        f3: "gamma".to_string(),
        f4: "delta".to_string(),
        f5: "epsilon".to_string(),
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        b1: true,
        b2: false,
        d1: std::f64::consts::PI,
        d2: std::f64::consts::E,
        opt: Some("present".to_string()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Serialization
// ─────────────────────────────────────────────────────────────────────────────

/// Hot path: serialize a small 3-field struct repeatedly.
#[test]
fn prof_serialize_simple() {
    let data = make_simple();
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&data));
    }
    // Prevent the compiler from eliminating the loop.
    assert!(!result.is_empty());
    eprintln!("[prof_serialize_simple] last output: {result}");
}

/// Hot path: serialize a moderately complex struct with nested types.
#[test]
fn prof_serialize_complex() {
    let data = make_complex();
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&data));
    }
    assert!(!result.is_empty());
    eprintln!("[prof_serialize_complex] last output length: {}", result.len());
}

/// Hot path: struct with 15 fields of mixed types.
#[test]
fn prof_serialize_many_fields() {
    let data = make_many_fields();
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&data));
    }
    assert!(!result.is_empty());
}

/// Hot path: pretty-print serialization (indented output).
#[test]
fn prof_serialize_pretty() {
    let data = make_complex();
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string_pretty(black_box(&data));
    }
    assert!(result.contains('\n'));
}

/// Hot path: serialize a large integer array (1 000 elements).
#[test]
fn prof_serialize_large_array() {
    let data: Vec<i64> = (0..1_000).map(|i| i * 7 - 3).collect();
    let mut result = String::new();
    for _ in 0..(ITERS / 10) {
        result = to_string(black_box(&data));
    }
    assert!(!result.is_empty());
    eprintln!("[prof_serialize_large_array] JSON length: {}", result.len());
}

/// Hot path: serialize a string that contains many escape sequences.
#[test]
fn prof_serialize_string_with_escapes() {
    let s = "hello\nworld\t\"escaped\"\r\nand\\more\x00control\x1Fchars".to_string();
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&s));
    }
    assert!(result.contains("\\n"));
}

/// Hot path: serialize a long string with NO escapes (fast path).
#[test]
fn prof_serialize_string_no_escapes() {
    let s = "abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789".repeat(10);
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&s));
    }
    assert!(!result.is_empty());
}

/// Hot path: serialize a BTreeMap.
#[test]
fn prof_serialize_btreemap() {
    let mut m: BTreeMap<String, i64> = BTreeMap::new();
    for i in 0..20i64 {
        m.insert(format!("key_{i:02}"), i * 100);
    }
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&m));
    }
    assert!(!result.is_empty());
}

/// Hot path: serialize enum variants (unit, tuple, struct).
#[test]
fn prof_serialize_enum_variants() {
    let variants: Vec<Event> = vec![
        Event::Click { x: 1.5, y: 2.5 },
        Event::Key("Enter".to_string()),
        Event::Resize(1920, 1080),
    ];
    let mut result = String::new();
    for _ in 0..ITERS {
        for v in black_box(&variants) {
            result = to_string(v);
        }
    }
    assert!(!result.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Deserialization
// ─────────────────────────────────────────────────────────────────────────────

/// Hot path: deserialize a small struct.
#[test]
fn prof_deserialize_simple() {
    let json = to_string(&make_simple());
    let mut result: Option<SimpleStruct> = None;
    for _ in 0..ITERS {
        result = Some(from_str::<SimpleStruct>(black_box(&json)).unwrap());
    }
    assert!(result.is_some());
    eprintln!("[prof_deserialize_simple] input: {json}");
}

/// Hot path: deserialize a moderately complex struct.
#[test]
fn prof_deserialize_complex() {
    let json = to_string(&make_complex());
    let mut result: Option<ComplexStruct> = None;
    for _ in 0..ITERS {
        result = Some(from_str::<ComplexStruct>(black_box(&json)).unwrap());
    }
    assert!(result.is_some());
    eprintln!("[prof_deserialize_complex] input length: {}", json.len());
}

/// Hot path: deserialize a struct with 15 fields.
#[test]
fn prof_deserialize_many_fields() {
    let json = to_string(&make_many_fields());
    let mut result: Option<ManyFields> = None;
    for _ in 0..ITERS {
        result = Some(from_str::<ManyFields>(black_box(&json)).unwrap());
    }
    assert!(result.is_some());
}

/// Hot path: deserialize a large integer array.
#[test]
fn prof_deserialize_large_array() {
    let data: Vec<i32> = (0..1_000).collect();
    let json = to_string(&data);
    let mut result: Option<Vec<i32>> = None;
    for _ in 0..(ITERS / 10) {
        result = Some(from_str::<Vec<i32>>(black_box(&json)).unwrap());
    }
    assert_eq!(result.unwrap().len(), 1_000);
}

/// Hot path: deserialize a string that exercises the escape decoder.
#[test]
fn prof_deserialize_string_with_escapes() {
    let original = "hello\nworld\t\"escaped\"\r\nand\\more";
    let json = to_string(&original);
    let mut result = String::new();
    for _ in 0..ITERS {
        result = from_str::<String>(black_box(&json)).unwrap();
    }
    assert_eq!(result, original);
}

/// Hot path: deserialize from raw bytes (UTF-8 validation + parse).
#[test]
fn prof_deserialize_from_bytes() {
    let json_bytes = to_string(&make_complex()).into_bytes();
    let mut result: Option<ComplexStruct> = None;
    for _ in 0..ITERS {
        result = Some(from_bytes::<ComplexStruct>(black_box(&json_bytes)).unwrap());
    }
    assert!(result.is_some());
}

/// Hot path: deserialize JSON that has extra unknown fields to skip.
#[test]
fn prof_deserialize_skip_unknown_fields() {
    let json = r#"{
        "name": "json-steroids",
        "extra_string": "should be ignored",
        "extra_object": {"a": 1, "b": [2, 3, 4]},
        "value": 42,
        "extra_array": [true, false, null],
        "active": true
    }"#;
    let mut result: Option<SimpleStruct> = None;
    for _ in 0..ITERS {
        result = Some(from_str::<SimpleStruct>(black_box(json)).unwrap());
    }
    assert!(result.is_some());
}

/// Hot path: deserialize enum variants.
#[test]
fn prof_deserialize_enum_variants() {
    let jsons = [
        to_string(&Event::Click { x: 1.5, y: 2.5 }),
        to_string(&Event::Key("Enter".to_string())),
        to_string(&Event::Resize(1920, 1080)),
        to_string(&Status::Active),
        to_string(&Status::Pending),
    ];
    let mut count = 0usize;
    for _ in 0..ITERS {
        for j in black_box(&jsons) {
            // parse as dynamic value to cover all variant shapes
            let _ = parse(j).unwrap();
            count += 1;
        }
    }
    assert_eq!(count, ITERS * jsons.len());
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Full round-trip
// ─────────────────────────────────────────────────────────────────────────────

/// Hot path: serialize then immediately deserialize (simple).
#[test]
fn prof_roundtrip_simple() {
    let data = make_simple();
    for _ in 0..ITERS {
        let json = to_string(black_box(&data));
        let rt = from_str::<SimpleStruct>(black_box(&json)).unwrap();
        let _ = black_box(rt);
    }
}

/// Hot path: serialize then immediately deserialize (complex).
#[test]
fn prof_roundtrip_complex() {
    let data = make_complex();
    for _ in 0..(ITERS / 2) {
        let json = to_string(black_box(&data));
        let rt = from_str::<ComplexStruct>(black_box(&json)).unwrap();
        let _ = black_box(rt);
    }
}

/// Hot path: round-trip a nested Vec<ComplexStruct> payload.
#[test]
fn prof_roundtrip_vec_of_structs() {
    let data: Vec<ComplexStruct> = (0..10).map(|_| make_complex()).collect();
    let mut last_len = 0usize;
    for _ in 0..(ITERS / 20) {
        let json = to_string(black_box(&data));
        let rt = from_str::<Vec<ComplexStruct>>(black_box(&json)).unwrap();
        last_len = rt.len();
    }
    assert_eq!(last_len, 10);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Dynamic (JsonValue) parsing
// ─────────────────────────────────────────────────────────────────────────────

/// Hot path: parse a mixed JSON object into a dynamic JsonValue.
#[test]
fn prof_parse_dynamic_mixed() {
    let json = r#"{"id":1,"name":"Alice","scores":[95,87,92],"meta":{"active":true,"ratio":0.98}}"#;
    let mut result: Option<JsonValue> = None;
    for _ in 0..ITERS {
        result = Some(parse(black_box(json)).unwrap());
    }
    assert!(result.unwrap().is_object());
}

/// Hot path: parse a deeply nested dynamic value (32 levels).
#[test]
fn prof_parse_dynamic_deeply_nested() {
    let open: String = "{\"x\":".repeat(32);
    let json = format!("{}42{}", open, "}".repeat(32));
    let mut result: Option<JsonValue> = None;
    for _ in 0..ITERS {
        result = Some(parse(black_box(&json)).unwrap());
    }
    assert!(result.is_some());
}

/// Hot path: parse a large JSON array of objects dynamically.
#[test]
fn prof_parse_dynamic_large_object_array() {
    // Build once, parse many times
    let items: Vec<String> = (0..100)
        .map(|i| format!(r#"{{"id":{i},"name":"item_{i}","value":{}}}"#, i * 10))
        .collect();
    let json = format!("[{}]", items.join(","));

    let mut len = 0usize;
    for _ in 0..(ITERS / 10) {
        let v = parse(black_box(&json)).unwrap();
        len = v.as_array().unwrap().len();
    }
    assert_eq!(len, 100);
}

/// Hot path: serialize a dynamic JsonValue back to a string.
#[test]
fn prof_serialize_dynamic_value() {
    let json = r#"{"key":"value","nums":[1,2,3,4,5],"flag":true,"nested":{"a":1}}"#;
    let value = parse(json).unwrap();
    let mut result = String::new();
    for _ in 0..ITERS {
        result = to_string(black_box(&value));
    }
    assert!(!result.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. JsonWriter direct API
// ─────────────────────────────────────────────────────────────────────────────

/// Hot path: build a JSON object manually via JsonWriter.
#[test]
fn prof_writer_manual_object() {
    let mut result = String::new();
    for _ in 0..ITERS {
        let mut w = JsonWriter::new();
        w.begin_object();
        w.write_key("id");
        w.write_i64(black_box(42));
        w.write_comma();
        w.write_key("name");
        w.write_string(black_box("Alice"));
        w.write_comma();
        w.write_key("score");
        w.write_f64(black_box(99.5));
        w.write_comma();
        w.write_key("active");
        w.write_bool(black_box(true));
        w.end_object();
        result = w.into_string();
    }
    assert!(result.contains("Alice"));
}

/// Hot path: build a JSON array of 100 integers via JsonWriter.
#[test]
fn prof_writer_large_integer_array() {
    let nums: Vec<i64> = (0..100).collect();
    let mut result = String::new();
    for _ in 0..ITERS {
        let mut w = JsonWriter::with_capacity(512);
        w.begin_array();
        for (i, &n) in black_box(&nums).iter().enumerate() {
            if i > 0 {
                w.write_comma();
            }
            w.write_i64(n);
        }
        w.end_array();
        result = w.into_string();
    }
    assert!(result.starts_with('['));
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Number-heavy payloads
// ─────────────────────────────────────────────────────────────────────────────

/// Hot path: round-trip a Vec<f64> with varied magnitudes.
#[test]
fn prof_roundtrip_floats() {
    let data: Vec<f64> = (0..500)
        .map(|i| (i as f64).powi(2) * 0.001 - 1234.5678)
        .collect();
    let mut count = 0usize;
    for _ in 0..(ITERS / 10) {
        let json = to_string(black_box(&data));
        let rt = from_str::<Vec<f64>>(black_box(&json)).unwrap();
        count = rt.len();
    }
    assert_eq!(count, 500);
}

/// Hot path: round-trip extreme integer values.
#[test]
fn prof_roundtrip_integer_extremes() {
    let data: Vec<i64> = vec![
        i64::MIN, i64::MIN / 2, -1, 0, 1, i64::MAX / 2, i64::MAX,
    ];
    for _ in 0..ITERS {
        let json = to_string(black_box(&data));
        let rt = from_str::<Vec<i64>>(black_box(&json)).unwrap();
        let _ = black_box(rt);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Throughput stress
// ─────────────────────────────────────────────────────────────────────────────

/// Stress: serialize+deserialize 10 000-element integer array.
#[test]
fn prof_stress_large_int_array() {
    let data: Vec<i32> = (0..10_000).collect();
    for _ in 0..100 {
        let json = to_string(black_box(&data));
        let rt = from_str::<Vec<i32>>(black_box(&json)).unwrap();
        assert_eq!(rt.len(), 10_000);
    }
}

/// Stress: 100 000-character string with no escapes (memcpy hot path).
#[test]
fn prof_stress_large_plain_string() {
    let s = "x".repeat(100_000);
    for _ in 0..100 {
        let json = to_string(black_box(&s));
        let rt = from_str::<String>(black_box(&json)).unwrap();
        assert_eq!(rt.len(), 100_000);
    }
}

/// Stress: alternating-escape string (every other char is `\n`).
#[test]
fn prof_stress_large_escape_string() {
    let s: String = (0..5_000).map(|i| if i % 2 == 0 { 'x' } else { '\n' }).collect();
    for _ in 0..100 {
        let json = to_string(black_box(&s));
        let rt = from_str::<String>(black_box(&json)).unwrap();
        assert_eq!(rt.len(), s.len());
    }
}

/// Stress: parse a realistic JSON payload resembling an API response.
#[test]
fn prof_stress_realistic_api_payload() {
    let json = r#"{
        "status": "ok",
        "code": 200,
        "data": {
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true, "score": 9.8},
                {"id": 2, "name": "Bob",   "email": "bob@example.com",   "active": false,"score": 7.2},
                {"id": 3, "name": "Carol", "email": "carol@example.com", "active": true, "score": 8.5},
                {"id": 4, "name": "Dave",  "email": "dave@example.com",  "active": true, "score": 6.1},
                {"id": 5, "name": "Eve",   "email": "eve@example.com",   "active": false,"score": 9.9}
            ],
            "total": 5,
            "page": 1,
            "per_page": 20
        },
        "meta": {
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "latency_ms": 12,
            "cached": false
        }
    }"#;

    let mut count = 0usize;
    for _ in 0..ITERS {
        let v = parse(black_box(json)).unwrap();
        count = v["data"]["users"].as_array().unwrap().len();
    }
    assert_eq!(count, 5);
}

