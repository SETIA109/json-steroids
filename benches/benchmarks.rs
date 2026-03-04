//! Benchmarks for json-steroids comparing with serde_json

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use json_steroids::{from_str, parse, to_string, Json};
use serde::{Deserialize, Serialize};

// ============ json-steroids types ============

#[derive(Debug, PartialEq, Json)]
struct SimpleStruct {
    name: String,
    value: i64,
    active: bool,
}

#[derive(Debug, PartialEq, Json)]
struct ComplexStruct {
    id: u64,
    name: String,
    tags: Vec<String>,
    metadata: Metadata,
    values: Vec<i32>,
}

#[derive(Debug, PartialEq, Json)]
struct Metadata {
    created: String,
    updated: String,
    version: u32,
}

// ============ serde types (identical structure) ============

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SerdeSimpleStruct {
    name: String,
    value: i64,
    active: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SerdeComplexStruct {
    id: u64,
    name: String,
    tags: Vec<String>,
    metadata: SerdeMetadata,
    values: Vec<i32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SerdeMetadata {
    created: String,
    updated: String,
    version: u32,
}

// ============ Data creation functions ============

fn create_simple_data() -> SimpleStruct {
    SimpleStruct {
        name: "test".to_string(),
        value: 42,
        active: true,
    }
}

fn create_serde_simple_data() -> SerdeSimpleStruct {
    SerdeSimpleStruct {
        name: "test".to_string(),
        value: 42,
        active: true,
    }
}

fn create_complex_data() -> ComplexStruct {
    ComplexStruct {
        id: 12345,
        name: "Complex Test Object".to_string(),
        tags: vec!["rust".to_string(), "json".to_string(), "fast".to_string()],
        metadata: Metadata {
            created: "2024-01-01T00:00:00Z".to_string(),
            updated: "2024-01-02T12:30:00Z".to_string(),
            version: 1,
        },
        values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    }
}

fn create_serde_complex_data() -> SerdeComplexStruct {
    SerdeComplexStruct {
        id: 12345,
        name: "Complex Test Object".to_string(),
        tags: vec!["rust".to_string(), "json".to_string(), "fast".to_string()],
        metadata: SerdeMetadata {
            created: "2024-01-01T00:00:00Z".to_string(),
            updated: "2024-01-02T12:30:00Z".to_string(),
            version: 1,
        },
        values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    }
}

// ============ Comparison benchmarks ============

fn bench_serialize_simple_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_simple");

    let steroids_data = create_simple_data();
    let serde_data = create_serde_simple_data();

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&steroids_data)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_data)).unwrap())
    });

    group.finish();
}

fn bench_deserialize_simple_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_simple");

    let json = r#"{"name":"test","value":42,"active":true}"#;

    group.bench_function("json-steroids", |b| {
        b.iter(|| from_str::<SimpleStruct>(black_box(json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<SerdeSimpleStruct>(black_box(json)))
    });

    group.finish();
}

fn bench_serialize_complex_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_complex");

    let steroids_data = create_complex_data();
    let serde_data = create_serde_complex_data();

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&steroids_data)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_data)).unwrap())
    });

    group.finish();
}

fn bench_deserialize_complex_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_complex");

    let steroids_data = create_complex_data();
    let json = to_string(&steroids_data);

    group.bench_function("json-steroids", |b| {
        b.iter(|| from_str::<ComplexStruct>(black_box(&json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<SerdeComplexStruct>(black_box(&json)))
    });

    group.finish();
}

fn bench_roundtrip_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_complex");

    let steroids_data = create_complex_data();
    let serde_data = create_serde_complex_data();

    group.bench_function("json-steroids", |b| {
        b.iter(|| {
            let json = to_string(black_box(&steroids_data));
            from_str::<ComplexStruct>(&json)
        })
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&serde_data)).unwrap();
            serde_json::from_str::<SerdeComplexStruct>(&json)
        })
    });

    group.finish();
}

fn bench_parse_dynamic_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_dynamic");

    let json = r#"{"name":"test","values":[1,2,3,4,5],"nested":{"a":true,"b":false}}"#;

    group.bench_function("json-steroids", |b| b.iter(|| parse(black_box(json))));

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json)))
    });

    group.finish();
}

fn bench_large_array_comparison(c: &mut Criterion) {
    #[derive(Debug, Serialize, Deserialize)]
    struct SerdeArrayWrapper {
        items: Vec<i32>,
    }

    #[derive(Debug, Json)]
    struct JsonSteroidsArrayWrapper {
        items: Vec<i32>,
    }

    let data: Vec<i32> = (0..1000).collect();
    let json_steroids_data = JsonSteroidsArrayWrapper {
        items: data.clone(),
    };
    let serde_data = SerdeArrayWrapper {
        items: data.clone(),
    };

    let mut group = c.benchmark_group("large_array_serialize");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&json_steroids_data)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_data)).unwrap())
    });

    group.finish();

    let json = to_string(&json_steroids_data);

    let mut group = c.benchmark_group("large_array_deserialize");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("json-steroids", |b| {
        b.iter(|| from_str::<JsonSteroidsArrayWrapper>(black_box(&json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<SerdeArrayWrapper>(black_box(&json)))
    });

    group.finish();
}

fn bench_string_escaping_comparison(c: &mut Criterion) {
    let simple = "hello world simple string without any escapes needed";
    let escaped = "hello\nworld\twith\rescapes\"and\\slashes\u{0000}";

    let mut group = c.benchmark_group("string_serialize_no_escapes");

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&simple)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&simple)).unwrap())
    });

    group.finish();

    let mut group = c.benchmark_group("string_serialize_with_escapes");

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&escaped)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&escaped)).unwrap())
    });

    group.finish();
}

fn bench_deeply_nested_comparison(c: &mut Criterion) {
    // Create a deeply nested JSON structure
    let json = r#"{
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "level5": {
                            "value": 42,
                            "array": [1, 2, 3, 4, 5],
                            "string": "deeply nested value"
                        }
                    }
                }
            }
        }
    }"#;

    let mut group = c.benchmark_group("deeply_nested_parse");

    group.bench_function("json-steroids", |b| b.iter(|| parse(black_box(json))));

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json)))
    });

    group.finish();
}

fn bench_many_fields_comparison(c: &mut Criterion) {
    #[derive(Debug, Json)]
    struct ManyFields {
        field1: String,
        field2: String,
        field3: String,
        field4: String,
        field5: String,
        field6: i32,
        field7: i32,
        field8: i32,
        field9: i32,
        field10: i32,
        field11: bool,
        field12: bool,
        field13: f64,
        field14: f64,
        field15: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct SerdeManyFields {
        field1: String,
        field2: String,
        field3: String,
        field4: String,
        field5: String,
        field6: i32,
        field7: i32,
        field8: i32,
        field9: i32,
        field10: i32,
        field11: bool,
        field12: bool,
        field13: f64,
        field14: f64,
        field15: Option<String>,
    }

    let steroids_data = ManyFields {
        field1: "value1".to_string(),
        field2: "value2".to_string(),
        field3: "value3".to_string(),
        field4: "value4".to_string(),
        field5: "value5".to_string(),
        field6: 100,
        field7: 200,
        field8: 300,
        field9: 400,
        field10: 500,
        field11: true,
        field12: false,
        field13: 3.14159,
        field14: 2.71828,
        field15: Some("optional".to_string()),
    };

    let serde_data = SerdeManyFields {
        field1: "value1".to_string(),
        field2: "value2".to_string(),
        field3: "value3".to_string(),
        field4: "value4".to_string(),
        field5: "value5".to_string(),
        field6: 100,
        field7: 200,
        field8: 300,
        field9: 400,
        field10: 500,
        field11: true,
        field12: false,
        field13: 3.14159,
        field14: 2.71828,
        field15: Some("optional".to_string()),
    };

    let mut group = c.benchmark_group("many_fields_serialize");

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&steroids_data)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_data)).unwrap())
    });

    group.finish();

    let json = to_string(&steroids_data);

    let mut group = c.benchmark_group("many_fields_deserialize");

    group.bench_function("json-steroids", |b| {
        b.iter(|| from_str::<ManyFields>(black_box(&json)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<SerdeManyFields>(black_box(&json)))
    });

    group.finish();
}

fn bench_numbers_comparison(c: &mut Criterion) {
    // Integer serialization
    let integers: Vec<i64> = (-500..500).collect();

    let mut group = c.benchmark_group("integers_serialize");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&integers)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&integers)).unwrap())
    });

    group.finish();

    // Float serialization
    let floats: Vec<f64> = (0..1000).map(|i| i as f64 * 0.123456789).collect();

    let mut group = c.benchmark_group("floats_serialize");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("json-steroids", |b| {
        b.iter(|| to_string(black_box(&floats)))
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&floats)).unwrap())
    });

    group.finish();
}

fn bench_cow_struct_comparison(c: &mut Criterion) {
    use std::borrow::Cow;

    // serde_json struct with borrowed fields (zero-copy capable)
    #[derive(Debug, Serialize, Deserialize)]
    struct SerdeUserDataBorrowed<'a> {
        username: Cow<'a, str>,
        email: Cow<'a, str>,
        full_name: Cow<'a, str>,
        bio: Cow<'a, str>,
    }

    // serde_json struct with borrowed fields (zero-copy capable)
    #[derive(Debug, Json)]
    struct JsonSteroidsUserDataBorrowed<'de> {
        username: Cow<'de, str>,
        email: Cow<'de, str>,
        full_name: Cow<'de, str>,
        bio: Cow<'de, str>,
    }

    // JSON with simple strings (no escape sequences - optimal for zero-copy)
    let json = r#"{"username":"alice_rust","email":"alice@example.com","full_name":"Alice Johnson","bio":"Rust enthusiast and open source contributor"}"#;

    let mut group = c.benchmark_group("cow_struct_deserialize");
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("json-steroids/deserialize borrowed (zero-copy)", |b| {
        b.iter(|| {
            let result: JsonSteroidsUserDataBorrowed = from_str(black_box(json)).unwrap();
            black_box(result)
        })
    });

    group.bench_function("serde_json/deserialize borrowed (zero-copy)", |b| {
        b.iter(|| {
            let result: SerdeUserDataBorrowed = serde_json::from_str(black_box(json)).unwrap();
            black_box(result)
        })
    });

    group.finish();

    let serde_borrowed_data = SerdeUserDataBorrowed {
        username: Cow::Borrowed("alice_rust"),
        email: Cow::Borrowed("alice@example.com"),
        full_name: Cow::Borrowed("Alice Johnson"),
        bio: Cow::Borrowed("Rust enthusiast and open source contributor"),
    };

    let json_steroids_borrowed_data = JsonSteroidsUserDataBorrowed {
        username: Cow::Borrowed("alice_rust"),
        email: Cow::Borrowed("alice@example.com"),
        full_name: Cow::Borrowed("Alice Johnson"),
        bio: Cow::Borrowed("Rust enthusiast and open source contributor"),
    };

    let mut group = c.benchmark_group("cow_struct_serialize");

    group.bench_function("json-steroids/serialize borrowed", |b| {
        b.iter(|| to_string(black_box(&json_steroids_borrowed_data)))
    });

    group.bench_function("serde_json/serialize borrowed", |b| {
        b.iter(|| serde_json::to_string(black_box(&serde_borrowed_data)).unwrap())
    });

    group.finish();
}

fn bench_borrowed_str_struct_comparison(c: &mut Criterion) {
    // serde_json struct with borrowed &str fields
    #[derive(Debug, Serialize, Deserialize)]
    struct SerdeUserBorrowed<'a> {
        username: &'a str,
        email: &'a str,
        role: &'a str,
    }

    // json-steroids struct with borrowed &str fields
    #[derive(Debug, Json)]
    struct JsonSteroidsUserBorrowed<'de> {
        username: &'de str,
        email: &'de str,
        role: &'de str,
    }

    // JSON with simple strings (no escape sequences - optimal for zero-copy)
    let json = r#"{"username":"john_doe","email":"john@example.com","role":"admin"}"#;

    let mut group = c.benchmark_group("borrowed_str_struct_deserialize");
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("json-steroids", |b| {
        b.iter(|| {
            let result: JsonSteroidsUserBorrowed = from_str(black_box(json)).unwrap();
            black_box(result)
        })
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| {
            let result: SerdeUserBorrowed = serde_json::from_str(black_box(json)).unwrap();
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_serialize_simple_comparison,
    bench_deserialize_simple_comparison,
    bench_serialize_complex_comparison,
    bench_deserialize_complex_comparison,
    bench_roundtrip_comparison,
    bench_parse_dynamic_comparison,
    bench_large_array_comparison,
    bench_string_escaping_comparison,
    bench_deeply_nested_comparison,
    bench_many_fields_comparison,
    bench_numbers_comparison,
    bench_cow_struct_comparison,
    bench_borrowed_str_struct_comparison,
);

criterion_main!(benches);
